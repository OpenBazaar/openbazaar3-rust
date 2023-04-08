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

    Ok(())
}
