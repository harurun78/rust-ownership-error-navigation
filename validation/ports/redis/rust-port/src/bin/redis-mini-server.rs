use std::env;

use rust_port::RedisMiniServer;

fn main() -> std::io::Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6379".to_string());
    RedisMiniServer::serve(addr)
}
