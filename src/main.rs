use std::net::{Ipv4Addr, SocketAddrV4};

use axum::{routing::post, Router};
use tokio::net::TcpListener;
use tracing::info;

use server::{MOS6502_ENDPOINT, RISC_V_ENDPOINT, X86_ENDPOINT};

mod server;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    // such separation allows to introduce conflicting and target-specific options if needed
    let routes = Router::new()
        .route(MOS6502_ENDPOINT, post(server::handle_mos6502))
        .route(X86_ENDPOINT, post(server::handle_x86))
        .route(RISC_V_ENDPOINT, post(server::handle_risc_v));

    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9999);
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("{:<15} - {:?}\n", "LISTENING", listener.local_addr());

    axum::serve(listener, routes.into_make_service())
        .await
        .unwrap();
}
