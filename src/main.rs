use http_types::Body;
use std::str::FromStr;
use tera::Tera;
use tide::http::Mime;
use tide::Request;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let tera = Tera::new("templates/**/*")?;

    let mut app = tide::with_state(tera);
    app.at("/:name").get(hello);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn hello(req: Request<Tera>) -> tide::Result {
    let tera = req.state();
    let mut response = tide::Response::new(200);
    let mut context = tera::Context::new();
    let name: String = req.param("name")?.to_string();
    context.insert("name", &name);

    let mut body = Body::from_string(tera.render("hello.html", &context)?);

    let mime = Mime::from_str("text/html;charset=utf-8").unwrap();
    body.set_mime(mime);
    response.set_body(body);

    Ok(response)
}
