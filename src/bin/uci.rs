fn main() {
    env_logger::init();
    log::info!("chess-uci v{}", env!("APP_VERSION"));
    let mut engine = chess::engine::Engine::new();
    chess::uci::run(&mut engine);
}
