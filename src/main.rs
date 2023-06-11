
mod error;
mod resolver;
mod util;

#[tokio::main]
async fn main() {
    resolver::run("0.0.0.0", 3053).await.expect("Server error");
}

