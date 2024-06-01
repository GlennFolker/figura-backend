use std::{
    fs::File,
    io::BufReader,
    num::ParseIntError,
    path::PathBuf,
    time::Duration,
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
    #[arg(short, long, default_value = "key.pem")]
    key: PathBuf,
    /// The PKCS#8-encoded public key file in PEM format with RSA algorithm.
    #[arg(short, long, default_value = "cert.pem")]
    cert: PathBuf,
    #[arg(short, long, default_value_t = 443)]
    /// Which port the HTTP server listens to.
    port: u16,
    /// Server ID assignation verification timeout.
    #[arg(short, long, value_parser = duration_str, default_value = "10")]
    server_id_timeout: Duration,
    /// Access token validation timeout.
    #[arg(short, long, value_parser = duration_str, default_value = "600")]
    access_timeout: Duration,

    /// Mojang's official Minecraft session server.
    #[cfg(feature = "mojang")]
    #[arg(long, default_value = "https://sessionserver.mojang.com/session/minecraft/")]
    mojang_session_server: String,
    /// Request timeout for Mojang's official Minecraft session server, in seconds.
    #[cfg(feature = "mojang")]
    #[arg(long, value_parser = duration_str, default_value = "30")]
    mojang_session_timeout: Duration,

    /// Ely's unofficial Minecraft session server.
    #[cfg(feature = "ely")]
    #[arg(long, default_value = "https://authserver.ely.by/session")]
    ely_session_server: String,
    /// Request timeout for Ely's unofficial Minecraft session server, in seconds.
    #[cfg(feature = "ely")]
    #[arg(long, value_parser = duration_str, default_value = "30")]
    ely_session_timeout: Duration,
}

#[inline]
fn duration_str(arg: &str) -> Result<Duration, ParseIntError> {
    Ok(Duration::from_secs(arg.parse()?))
}

fn main() -> anyhow::Result<()> {
    System::new().block_on(async move {
        #[cfg(any(feature = "mojang", feature = "ely"))]
        use figura_auth_yggdrasil::YggdrasilConfig;

        env_logger::builder()
            .filter_level(LevelFilter::Info)
            .parse_default_env()
            .try_init()?;

        let args = match Args::try_parse() {
            Ok(args) => args,
            Err(e) => return Ok(e.print()?),
        };

        Backend {
            port: args.port,
            key: BufReader::new(File::open(args.key)?),
            cert: BufReader::new(File::open(args.cert)?),

            server_id_timeout: args.server_id_timeout,
            access_timeout: args.access_timeout,

            configs: vec![
                // The authentication stack prioritizes Mojang's Yggdrasil server first.
                #[cfg(feature = "mojang")]
                Box::new(YggdrasilConfig {
                    session_server: args.mojang_session_server,
                    timeout: args.mojang_session_timeout,
                }),
                #[cfg(feature = "ely")]
                Box::new(YggdrasilConfig {
                    session_server: args.ely_session_server,
                    timeout: args.ely_session_timeout,
                }),
            ],
        }
        .run()
        .await
    })
}
