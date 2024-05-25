use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
};

use clap::Parser;
use figura_api::{
    actix::System,
    anyhow,
    log::LevelFilter,
    Backend,
};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// The PKCS#8-encoded private key file in PEM format with RSA algorithm.
    #[arg(short, long, default_value_os = "key.pem")]
    key: PathBuf,
    /// The PKCS#8-encoded public key file in PEM format with RSA algorithm.
    #[arg(short, long, default_value_os = "cert.pem")]
    cert: PathBuf,
    #[arg(short, long, default_value_t = 443)]
    /// Which port the HTTP server listens to.
    port: u16,
    /// Mojang's official Minecraft session server.
    #[cfg(feature = "mojang")]
    #[arg(short, long, default_value = "https://sessionserver.mojang.com/session/minecraft/")]
    mojang_session_server: String,
    /// Ely's unofficial Minecraft session server.
    #[cfg(feature = "ely")]
    #[arg(short, long, default_value = "https://authserver.ely.by/session")]
    ely_session_server: String,
}

fn main() -> anyhow::Result<()> {
    System::new().block_on(async move {
        env_logger::builder()
            .filter_level(LevelFilter::Info)
            .parse_default_env()
            .try_init()?;

        let args = match Args::try_parse() {
            Ok(args) => args,
            Err(e) => return Ok(e.print()?),
        };

        let mut backend = Backend::new(
            args.port,
            BufReader::new(File::open(args.key)?),
            BufReader::new(File::open(args.cert)?),
        );

        // The authentication stack prioritizes Mojang's Yggdrasil server first.
        #[cfg(feature = "mojang")]
        {
            use figura_auth_yggdrasil::YggdrasilConfig;
            backend = backend.config(YggdrasilConfig {
                session_server: args.mojang_session_server,
            });
        }

        #[cfg(feature = "ely")]
        {
            use figura_auth_yggdrasil::YggdrasilConfig;
            backend = backend.config(YggdrasilConfig {
                session_server: args.ely_session_server,
            });
        }

        backend.run().await
    })
}
