mod network;
mod webserver;
mod wallet;

use clap::{Args, Parser, Subcommand};
use actix_web::{web, HttpRequest, Responder, HttpResponse};
use tracing::Level;


#[derive(Parser)]
#[command(name = "openbazaar3")]
#[command(about = "OpenBazaar 3 Marketplace", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command()] 
    Start {},
}

// TODO: Add configuration args for config file, ports, etc.
// #[derive(Args)]
// struct StartArgs {

// }

fn main() -> anyhow::Result<()> {
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Start { } => {

            let libp2p_port = 4002;
            let libp2p_ip4 = "0.0.0.0";
            let http_host = "0.0.0.0";
            let http_port = 8080;
            // TODO: implement TLS
            // let https_port = 8081;

            println!("Starting OpenBazaar..."); 

            // Set up tracing
            let collector = tracing_subscriber::fmt()
                .with_max_level(Level::INFO)
                .finish();
            tracing::subscriber::set_global_default(collector)
                .expect("There was a problem setting up tracing");
            
            // Create tokio async runtime
            let rt = tokio::runtime::Runtime::new().unwrap();

            /************
             * Set up libp2p network
             */

            // TODO: recover existing key instead of creating a new one each time
            let keypair = network::generate_key();

            // Create a new libp2p network and wait for it to spin up
            let (client, mut event_loop) = rt.block_on(async move {
                network::new(keypair).await.unwrap()
            });

            // Kick off the event loop handler in a thread
            let event_loop_handler = rt.spawn(async move {
                event_loop.run().await
            });
            
            // Fire up the network listener for incoming connections
            let mut client = client.clone();
            rt.block_on(async move {
                let addr = format!("/ip4/{}/tcp/{}", libp2p_ip4, libp2p_port).parse().expect("Failed to parse multiaddr");
                client.start_listening(addr).await.unwrap();
            });

            // TODO: Set up TLS connection
            
            // Fire up the web server for our API
            rt.spawn(async move { 
                let http_addr = format!("{}:{}", http_host, http_port);
                webserver::start_webserver(http_addr).await 
            });


            // TODO: Start up bitcoin wallet
            wallet::fire_up_wallet();

            println!("OpenBazaar started successfully! (Press Ctrl+C to exit)");

            let signal_handler = rt.spawn(async move {
                tokio::signal::ctrl_c().await.unwrap();
                event_loop_handler.abort();
            });

            rt.block_on(async move {
                tokio::signal::ctrl_c().await.unwrap();
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

