use crate::{entity::Entity, physics::Physics};
use crate::sprite::Sprite;

use winit::event::{ElementState, KeyboardInput};

pub struct Input {
    pub cursor: [f32; 2],
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl Input {
    pub fn new() -> Self {
        Input {
            cursor: [0.0, 0.0],
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }

    pub fn handle_input(&mut self, input: KeyboardInput) {
        match input.scancode {
            // Clockwise arrow keys
            103 | 17 => {
                self.up = match input.state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
            }
            106 | 32 => {
                self.right = match input.state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
            },
            108 | 31 => {
                self.down = match input.state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
            },
            105 | 30 => {
                self.left = match input.state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
            },
            _ => {},
        }
    }

    pub fn handle_movement(&self, player: &mut Entity, global: &Physics, factor: &[f32]) {
        let local = &mut player.physics;

        local.acceleration.0 += factor[0] * (0.0 + (self.right as i32 as f32) - (self.left as i32 as f32));
        local.acceleration.1 += factor[1] * (0.0 + (self.down as i32 as f32) - (self.up as i32 as f32));

        let resultant = Physics::resultant(local, global);

        let pos = resultant.get_position_delta();
        let norm_pos = Input::normalise_position(player.sprite.position.0 + pos.0, player.sprite.position.1 + pos.1, player.sprite.size);

        // Reset acceleration when collision with wall
        if norm_pos.0 != player.sprite.position.0 + pos.0 {
            local.acceleration.0 = 0.0;
        }
        if norm_pos.1 != player.sprite.position.1 + pos.1 {
            local.acceleration.1 = 0.0;
        }

        let mut new_sprite = Sprite::new(
            norm_pos,
            player.sprite.size,
            Some(player.sprite.shader),
        );
        new_sprite.texture = player.sprite.texture.clone();

        player.sprite = new_sprite;
    }

    pub fn is_valid_cursor_position(&self) -> bool {
        if self.cursor[0] < -1.0 || self.cursor[0] > 1.0 { return false; }
        if self.cursor[1] < -1.0 || self.cursor[1] > 1.0 { return false; }
        true
    }

    fn normalise_position(x: f32, y: f32, size: (f32, f32)) -> (f32, f32) {
        let bounds: ((f32, f32), (f32, f32)) = (
            (-1.0, 1.0 - size.0),
            (-1.0, 1.0 - size.1),
        );

        (
            match x {
                p if p < bounds.0.0 => bounds.0.0,
                p if p > bounds.0.1 => bounds.0.1,
                p => p,
            },
            match y {
                p if p < bounds.1.0 => bounds.1.0,
                p if p > bounds.1.1 => bounds.1.1,
                p => p,
            },
        )
    }
}