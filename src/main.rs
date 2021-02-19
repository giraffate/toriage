mod pulls;

use tera::Tera;
use tide::http::mime;
use tide::utils::After;
use tide::{Response, StatusCode};

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

    app.at("/pulls/:owner/:repo").get(pulls);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
