#[get("/")]
pub fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
pub async fn start_server() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![index])
        .launch()
        .await?;

    Ok(())
}
