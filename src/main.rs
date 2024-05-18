use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
};
use clap::Parser;
use log::LevelFilter;
use figura_backend::Backend;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// The PKCS#8-encoded private key file in PEM format with RSA algorithm.
    #[arg(short, long, default_value_os = "key.pem")]
    key: PathBuf,
    /// The PKCS#8-encoded public key file in PEM format with RSA algorithm.
    #[arg(short, long, default_value_os = "cert.pem")]
    cert: PathBuf,
    /// Which port the HTTP server listens to.
    port: u16,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .try_init()?;

    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(e) => return Ok(e.print()?),
    };

    Backend::new(args.port, &mut BufReader::new(File::open(args.key)?), &mut BufReader::new(File::open(args.cert)?)).run().await
}
