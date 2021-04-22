use chrono::{Duration, Utc};
use hyper::{Body, Request, Response};
use keiro::Handler;
use serde::Serialize;
use serde_json::value::{to_value, Value};
use std::future::Future;
use std::pin::Pin;

const YELLOW_DAYS: i64 = 7;
const RED_DAYS: i64 = 14;

pub struct PullsHandler {
    pub tera: tera::Tera,
    pub token: String,
}

impl Handler for PullsHandler {
    fn call(
        &self,
        req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = keiro::Result<Response<Body>>> + Send + Sync>> {
        let params = req.extensions().get::<keiro::Params>().unwrap();
        let owner = params.find("owner").unwrap();
        let repo = params.find("repo").unwrap();

        if &owner[..] != "rust-lang" {
            let response = Response::builder()
                .status(404)
                .header("Content-Type", "text/html")
                .body(Body::from(include_str!("../templates/404.html")))
                .unwrap();

            return Box::pin(async { Ok(response) });
        }

        let token = self.token.clone();

        let dummy_root = url::Url::parse("file://").unwrap();
        let url = dummy_root.join(&req.uri().to_string()).unwrap();
        let query_params = url.query_pairs();
        let query_params: std::collections::HashMap<String, String> = query_params
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let page_param = match query_params.get(&"page".to_string()) {
            Some(p) => p.parse::<u8>().unwrap(),
            None => 1,
        };

        let client = ghrs::Client::new();
        let mut page: ghrs::Page<ghrs::model::PullRequest> = client
            .token(token)
            .pulls(owner.clone(), repo.clone())
            .list()
            .per_page(100)
            .page(page_param)
            .sort("updated")
            .direction("asc")
            .send()
            .unwrap();
        let base_pulls = page.take_items();

        let mut pulls: Vec<Value> = Vec::new();
        for base_pull in base_pulls.into_iter() {
            let assignee = base_pull.assignee.map_or("".to_string(), |v| v.login);
            let updated_at = base_pull
                .updated_at
                .map_or("".to_string(), |v| v.to_rfc2822());

            let yellow_line = Utc::now() - Duration::days(YELLOW_DAYS);
            let red_line = Utc::now() - Duration::days(RED_DAYS);
            let need_triage = match base_pull.updated_at {
                Some(updated_at) if updated_at <= red_line => "red".to_string(),
                Some(updated_at) if updated_at <= yellow_line => "yellow".to_string(),
                _ => "green".to_string(),
            };

            let mut labels = "".to_string();
            let mut wait_for_author = false;
            let mut wait_for_review = false;
            if !base_pull.labels.is_empty() {
                labels = base_pull
                    .labels
                    .iter()
                    .map(|label| label.name.clone())
                    .collect::<Vec<_>>()
                    .join(",");
                wait_for_author = labels.contains("S-waiting-on-author");
                wait_for_review = labels.contains("S-waiting-on-review");
            }

            let pull = PullRequest {
                html_url: base_pull.html_url,
                number: base_pull.number,
                title: base_pull.title,
                assignee,
                updated_at,
                need_triage,
                labels,
                author: base_pull.user.login,
                wait_for_author,
                wait_for_review,
            };
            pulls.push(to_value(pull).unwrap());
        }

        let mut context = tera::Context::new();
        context.insert("pulls", &pulls);
        context.insert("owner", &owner);
        context.insert("repo", &repo);

        if let Some(_) = page.get_prev() {
            let dummy_root = url::Url::parse("file://").unwrap();
            let mut url = dummy_root.join(&req.uri().to_string()).unwrap();
            {
                let mut query_params = url.query_pairs_mut();
                query_params.append_pair("page", &(page_param - 1).to_string());
            }

            match url.query() {
                Some(query) => context.insert("prev", &(format!("{}?{}", url.path(), query))),
                None => {}
            }
        }
        if let Some(_) = page.get_next() {
            let dummy_root = url::Url::parse("file://").unwrap();
            let mut url = dummy_root.join(&req.uri().to_string()).unwrap();
            {
                let mut query_params = url.query_pairs_mut();
                query_params.append_pair("page", &(page_param + 1).to_string());
            }

            match url.query() {
                Some(query) => context.insert("next", &(format!("{}?{}", url.path(), query))),
                None => {}
            }
        }

        let body = Body::from(self.tera.render("pulls.html", &context).unwrap());

        let response = Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(body)
            .unwrap();

        Box::pin(async { Ok(response) })
    }
}

#[derive(Serialize)]
struct PullRequest {
    pub html_url: String,
    pub number: u64,
    pub title: String,
    pub assignee: String,
    pub updated_at: String,
    pub need_triage: String,
    pub labels: String,
    pub author: String,
    pub wait_for_author: bool,
    pub wait_for_review: bool,
}
