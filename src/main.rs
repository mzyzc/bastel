use bastel;
use bastel::engine::Engine;

fn main() {
    const TITLE: &str = "BASTEL";
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    let (engine, event_loop) = Engine::new(TITLE, WIDTH, HEIGHT);
    bastel::begin_loop(engine, event_loop);
}