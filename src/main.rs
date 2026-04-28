mod engine;
mod uci;

fn main() {
    let mut engine = engine::Engine::new();
    uci::run(&mut engine);
}
