use crate::renderer::Renderer;
use crate::input::Input;
use crate::sprite::Sprite;

use winit::event_loop::EventLoop;

pub struct Engine {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fps: u64,
    pub renderer: Renderer,
    pub input: Input,
    pub sprites: Vec<Sprite>,
}

impl Engine {
    pub fn new(title: &str, width: u32, height: u32) -> (Self, EventLoop<()>) {
        let (renderer, event_loop) = Renderer::init(title, width, height);
        let input = Input::new();
        let fps = 60;

        (Engine {
            title: String::from(title),
            width,
            height,
            fps,
            renderer,
            input,
            sprites: vec![Sprite::new(
                (-1.0, -1.0),
                (2.0, 2.0),
                None,
            )],
        }, event_loop)
    }
}