mod network;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "openbazaar3")]
#[command(about = "OpenBazaar 3 Marketplace", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(arg_required_else_help = false)] 
    Start {},
}

#[derive(Args)]
struct StartArgs {

}

// const BOOTNODES: [&str; 1] = [
//     "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjBjUsASE"
// ];

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Start { } => {
            println!("Starting OpenBazaar 3..."); 

            // Create tokio async runtime
            let rt = tokio::runtime::Runtime::new().unwrap();

            // Add libp2p 
            let (client, mut event_loop) = rt.block_on(async move {
                network::new().await.unwrap()
            });

            let event_loop_handler = rt.spawn(async move {
                event_loop.run().await
            });

            let mut listener_client = client.clone();

            rt.block_on(async move {
                listener_client.start_listening("/ip4/0.0.0.0/tcp/0".parse().unwrap());
            });
            
            

            println!("Got here...");

            // Start up bitcoin wallet
            // TODO

        }

        
    }    

    Ok(())

}