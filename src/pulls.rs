use chrono::{Duration, Utc};
use http_types::Body;
use serde::{Deserialize, Serialize};
use serde_json::value::{to_value, Value};
use std::str::FromStr;
use tera::Tera;
use tide::http::Mime;
use tide::Request;

const YELLOW_DAYS: i64 = 7;
const RED_DAYS: i64 = 14;

pub async fn pulls(mut req: Request<Tera>) -> tide::Result {
    let owner: String = req.param("owner")?.to_string();
    let repo: String = req.param("repo")?.to_string();
    let params: Params = req.query()?;

    let client = ghrs::Client::new();
    let mut page: ghrs::Page<ghrs::model::PullRequest> = client
        .pulls(owner.clone(), repo.clone())
        .list()
        .per_page(100)
        .page(params.page)
        .sort("updated")
        .direction("asc")
        .send()?;
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
        pulls.push(to_value(pull)?);
    }

    let mut context = tera::Context::new();
    context.insert("pulls", &pulls);
    context.insert("owner", &owner);
    context.insert("repo", &repo);

    if let Some(_) = page.get_prev() {
        let prev_params = Params {
            page: params.page - 1,
        };
        let req: &mut tide::http::Request = req.as_mut();
        req.set_query(&prev_params)?;
        context.insert("prev", req.url());
    }
    if let Some(_) = page.get_next() {
        let next_params = Params {
            page: params.page + 1,
        };
        let req: &mut tide::http::Request = req.as_mut();
        req.set_query(&next_params)?;
        context.insert("next", req.url());
    }

    let tera = req.state();
    let mut body = Body::from_string(tera.render("pulls.html", &context)?);

    let mime = Mime::from_str("text/html;charset=utf-8").unwrap();
    body.set_mime(mime);

    let mut response = tide::Response::new(200);
    response.set_body(body);

    Ok(response)
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

#[derive(Deserialize, Serialize)]
#[serde(default)]
struct Params {
    page: u8,
}

impl Default for Params {
    fn default() -> Self {
        Self { page: 1 }
    }
}
