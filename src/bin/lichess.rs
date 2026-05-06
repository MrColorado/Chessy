use clap::Parser;

#[derive(Parser)]
#[command(name = "chess-lichess", version = env!("APP_VERSION"), about = "Lichess chess bot")]
struct Args {
    /// Lichess API token
    #[arg(long, env = "LICHESS_TOKEN")]
    token: String,
    /// Local HTTP control port
    #[arg(long, default_value = "8080")]
    port: u16,
    /// Maximum number of concurrent games
    #[arg(long, default_value = "1")]
    max_games: usize,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
    log::info!("chess-lichess {} on port {}", env!("APP_VERSION"), args.port);
    tokio::runtime::Runtime::new()?.block_on(chess::lichess::run(args.token, args.port, args.max_games))
}
