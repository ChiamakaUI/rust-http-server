use http_server::create_routes;


#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let app = create_routes();

    let port = std::env::var("PORT").unwrap_or_else(|_| "5000".to_string());
    let addr = format!("0.0.0.0:{}", port).parse().unwrap();
    println!("Server running on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
