fn main() {
    eprintln!("chess-uci v{}", env!("APP_VERSION"));
    let mut engine = chess::engine::Engine::new();
    chess::uci::run(&mut engine);
}
