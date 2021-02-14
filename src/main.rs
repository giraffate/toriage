mod pulls;

use tera::Tera;

use crate::pulls::pulls;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let tera = Tera::new("templates/**/*")?;

    let mut app = tide::with_state(tera);
    app.at("/pulls/:owner/:repo").get(pulls);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
