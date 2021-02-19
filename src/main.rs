mod pulls;

use http_types::Body;
use tera::Tera;
use tide::http::mime;
use tide::utils::After;
use tide::{Request, Response, StatusCode};

use crate::pulls::pulls;

#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::start();
    let tera = Tera::new("templates/**/*")?;

    let mut app = tide::with_state(tera);

    app.with(After(|response: Response| async move {
        let response = match response.status() {
            StatusCode::NotFound => Response::builder(404)
                .content_type(mime::HTML)
                .body(include_str!("../templates/404.html"))
                .build(),
            _ => response,
        };

        Ok(response)
    }));

    app.at("/").get(index);
    app.at("/pulls/:owner/:repo").get(pulls);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

pub async fn index(_req: Request<Tera>) -> tide::Result {
    let mut body = Body::from_string(include_str!("../templates/index.html").to_string());
    body.set_mime(mime::HTML);

    let mut response = tide::Response::new(200);
    response.set_body(body);

    Ok(response)
}
