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
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    eprintln!("chess-lichess v{} on port {}", env!("APP_VERSION"), args.port);
    tokio::runtime::Runtime::new()?.block_on(chess::lichess::run(args.token, args.port))
}
