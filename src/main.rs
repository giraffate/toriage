use chrono::{Duration, Utc};
use http_types::Body;
use serde::Serialize;
use serde_json::value::{to_value, Value};
use std::str::FromStr;
use tera::Tera;
use tide::http::Mime;
use tide::Request;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let tera = Tera::new("templates/**/*")?;

    let mut app = tide::with_state(tera);
    app.at("/pulls/:owner/:repo").get(pulls);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn pulls(req: Request<Tera>) -> tide::Result {
    let tera = req.state();
    let mut response = tide::Response::new(200);
    // let mut context = tera::Context::new();
    let owner: String = req.param("owner")?.to_string();
    let repo: String = req.param("repo")?.to_string();

    let client = ghrs::Client::new();
    let mut page = client.pulls(owner.clone(), repo.clone()).list().send()?;
    let base_pulls = page.take_items();
    let mut pulls: Vec<Value> = Vec::new();
    for base_pull in base_pulls.into_iter() {
        let assignee = if base_pull.assignee.is_some() {
            base_pull.assignee.unwrap().login
        } else {
            "".to_string()
        };

        let updated_at = if base_pull.updated_at.is_some() {
            base_pull.updated_at.unwrap().to_rfc2822()
        } else {
            "".to_string()
        };

        let need_triage = if base_pull.updated_at.is_none() {
            "lightgreen".to_string()
        } else {
            let yellow_line = Utc::now() - Duration::days(14);
            let red_line = Utc::now() - Duration::days(28);
            let updated_at = base_pull.updated_at.unwrap();

            if updated_at <= red_line {
                "red".to_string()
            } else if updated_at <= yellow_line {
                "yellow".to_string()
            } else {
                "lightgreen".to_string()
            }
        };

        let labels = if base_pull.labels.is_empty() {
            "".to_string()
        } else {
            base_pull
                .labels
                .iter()
                .map(|label| label.name.clone())
                .collect::<Vec<_>>()
                .join(",")
        };

        let pull = PullRequest {
            html_url: base_pull.html_url,
            number: base_pull.number,
            title: base_pull.title,
            assignee: assignee,
            updated_at: updated_at,
            need_triage: need_triage,
            labels: labels,
        };
        pulls.push(to_value(pull)?);
    }
    let mut context = tera::Context::new();
    context.insert("pulls", &pulls);
    context.insert("owner", &owner);
    context.insert("repo", &repo);

    let mut body = Body::from_string(tera.render("pulls.html", &context)?);

    let mime = Mime::from_str("text/html;charset=utf-8").unwrap();
    body.set_mime(mime);
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
}
