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
        let pull = PullRequest {
            html_url: base_pull.html_url,
            number: base_pull.number,
            title: base_pull.title,
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
}
