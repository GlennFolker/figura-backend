use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
};
use clap::Parser;
use figura_backend::Backend;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long, default_value_os = "./key.pem")]
    key: PathBuf,
    #[arg(short, long, default_value_os = "./cert.pem")]
    cert: PathBuf,
    /// Which port the HTTP server listens to.
    port: u16,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::try_init()?;
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(e) => return Ok(e.print()?),
    };

    Backend::new(args.port, BufReader::new(File::open(args.key)?), BufReader::new(File::open(args.cert)?)).run().await
}
