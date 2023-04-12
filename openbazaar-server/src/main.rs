mod api;
mod crypto;
mod db;
mod network;
mod profile;
mod wallet;
mod webserver;

use crate::openbazaar::open_bazaar_rpc_server::OpenBazaarRpcServer;
use crate::{
    api::OpenBazaarRpcService,
    db::{OpenBazaarDb, DB},
};
use actix_web::{http::Method, web, HttpRequest, HttpResponse, Responder};
use clap::{Parser, Subcommand};
use libp2p::{multiaddr::Protocol, Multiaddr, PeerId};
use std::{net::SocketAddr, path::PathBuf, str::FromStr};
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{Any, CorsLayer};
use tracing::Level;

pub mod openbazaar {
    include!(concat!(env!("OUT_DIR"), "/openbazaar_rpc.rs"));
}

#[derive(Parser)]
#[command(name = "openbazaar3")]
#[command(about = "OpenBazaar 3 Marketplace", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start the OpenBazaar server")]
    Start {
        #[arg(long, value_name = "LIBP2P_PORT")]
        libp2p_port: Option<u16>,

        #[arg(long, value_name = "LIBP2P_HOSTNAME")]
        libp2p_hostname: Option<String>,

        #[arg(long, value_name = "API_SERVER_PORT")]
        api_server_port: Option<u16>,

        #[arg(long, value_name = "API_SERVER_HOSTNAME")]
        api_server_hostname: Option<String>,

        #[arg(short, long, value_name = "USER")]
        user: Option<PathBuf>,

        #[arg(short, long, value_name = "GRPC_SERVER")]
        grpc_server: Option<SocketAddr>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            libp2p_port,
            libp2p_hostname,
            api_server_port,
            api_server_hostname,
            user,
            grpc_server,
        } => {
            println!("Starting OpenBazaar...");

            // Set up tracing
            let collector = tracing_subscriber::fmt()
                .with_max_level(Level::DEBUG)
                .finish();
            tracing::subscriber::set_global_default(collector)
                .expect("There was a problem setting up tracing");

            // Set up default server hostnames/ports
            let libp2p_hostname = libp2p_hostname.unwrap_or("0.0.0.0".to_string());
            let libp2p_port = libp2p_port.unwrap_or(4001);

            let http_host = api_server_hostname.unwrap_or("0.0.0.0".to_string());
            let http_port = api_server_port.unwrap_or(8080);

            let grpc_server = grpc_server.unwrap_or(SocketAddr::from_str("0.0.0.0:8010").unwrap());

            let data_directory = user.unwrap_or(PathBuf::from("data"));
            let db_file = format!("data/{}/openbazaar.db", data_directory.to_str().unwrap());

            // Create tokio async runtime
            let rt = tokio::runtime::Runtime::new().unwrap();

            // Create or retrieve datastore
            let ds = rt.block_on(async move { OpenBazaarDb::new(db_file).await.unwrap() });

            // Retrieve or create a new BIP39-based identity from the datastore
            let kp_ds = ds.clone();
            let keypair = rt.block_on(async move { kp_ds.get_identity().await.unwrap() });

            /************
             * Set up libp2p network
             */

            // Create a new libp2p network and wait for it to spin up
            let (client, mut event_loop) =
                rt.block_on(async move { network::new(keypair).await.unwrap() });

            // Kick off the event loop handler in a thread
            let event_loop_handler = rt.spawn(async move { event_loop.run().await });

            // Fire up the network listener for incoming connections
            let mut listener_client = client.clone();
            rt.block_on(async move {
                let addr = format!("/ip4/{}/tcp/{}", libp2p_hostname, libp2p_port)
                    .parse()
                    .expect("Failed to parse multiaddr");
                listener_client.start_listening(addr).await.unwrap();
            });

            // Connect to bootstrap node
            let mut client_dial = client.clone();
            if let Some(addr) = std::env::var_os("PEER") {
                let addr = Multiaddr::from_str(&addr.to_string_lossy())
                    .expect("Failed to parse multiaddr");
                let peer_id = match addr.iter().last() {
                    Some(Protocol::P2p(hash)) => {
                        PeerId::from_multihash(hash).expect("Failed to parse peer ID")
                    }
                    _ => panic!("No peer ID found in multiaddr"),
                };
                rt.block_on(async move {
                    client_dial.dial(peer_id, addr).await.unwrap();
                })
            }

            let mut peer_dial = client.clone();
            if let Some(addr) = std::env::var_os("PEER") {
                let addr = Multiaddr::from_str(addr.to_str().unwrap())
                    .expect("Couldn't parse multiaddress");
                let peer_id = match addr.iter().last() {
                    Some(Protocol::P2p(hash)) => PeerId::from_multihash(hash).expect("Valid hash."),
                    _ => panic!("Expect peer multiaddr to contain peer ID."),
                };
                rt.block_on(async move {
                    peer_dial
                        .dial(peer_id, addr)
                        .await
                        .expect("Dial to succeed");
                });
            }

            // TODO: Set up TLS connection

            // Fire up the web server for our API
            rt.spawn(async move {
                let http_addr = format!("{}:{}", http_host, http_port);
                webserver::start_webserver(http_addr).await
            });

            // TODO: Start up bitcoin wallet
            let wallet_ds = ds.clone();
            rt.block_on(async move {
                let mnemonic = wallet_ds.get_mnemonic().await.unwrap();
                wallet::fire_up_wallet(
                    mnemonic,
                    format!("data/{}", &data_directory.to_str().unwrap()),
                );
            });

            println!("\nOpenBazaar started successfully! (Press Ctrl+C to exit)");

            let signal_handler = rt.spawn(async move {
                tokio::signal::ctrl_c().await.unwrap();
                event_loop_handler.abort();
            });

            // Construct OpenBazaar service
            let ob_service = OpenBazaarRpcService::new(client.clone(), ds);

            let tonic_server = Server::builder();

            let cors = CorsLayer::new()
                // allow any headers
                .allow_headers(Any)
                // allow `GET` when accessing the resource
                .allow_methods([Method::GET, Method::POST])
                // allow requests from below origins
                .allow_origin([
                    "http://localhost:3000".parse()?,
                    "https://localhost:3001".parse()?,
                ]);

            let tonic_server_handler = rt.spawn(async move {
                tonic_server
                    .accept_http1(true)
                    .layer(cors)
                    .layer(GrpcWebLayer::new())
                    .add_service(OpenBazaarRpcServer::new(ob_service))
                    .serve(grpc_server)
                    .await
                    .unwrap();
            });

            println!("gRPC server listening on {}", grpc_server);

            rt.block_on(async move {
                tokio::signal::ctrl_c().await.unwrap();
                tonic_server_handler.abort();
                signal_handler.abort();
            });
        }
    }

    Ok(())
}

pub struct OBData {
    count: std::cell::Cell<usize>,
}

pub async fn handler(req: HttpRequest, counter: web::Data<OBData>) -> impl Responder {
    // note this cannot use the Data<T> extractor because it was not added with it
    let incr = *req.app_data::<usize>().unwrap();
    assert_eq!(incr, 3);

    // update counter using other value from app data
    counter.count.set(counter.count.get() + incr);

    HttpResponse::Ok().body(counter.count.get().to_string())
}
