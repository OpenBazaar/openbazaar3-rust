use dialoguer::{theme::ColorfulTheme, Select};
use openbazaar_lib::Client;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, default_value = "http://127.0.0.1:8010")]
    server: String,
    #[structopt(parse(from_os_str))]
    db_path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let mut client = Client::new(opt.db_path).unwrap();

    println!("Connecting to server: {}", opt.server);
    client.connect(opt.server)?;

    loop {
        let options = vec!["Exit"];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an option")
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                println!("Exiting...");
                break;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
