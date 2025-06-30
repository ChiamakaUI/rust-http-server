use http_server::create_routes;


#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let app = create_routes();

    let addr = "0.0.0.0:5000".parse().unwrap();
    println!("Server running on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
