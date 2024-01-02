use std::time::Instant;

use shared::glam::Vec3;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};

pub struct OffsetEntity {
    pub offset_parameter: Vec3,
    offset_velocity: Vec3,
    last_update: Instant,
}

impl OffsetEntity {
    const BOUNCE: f32 = 1.0;
    const SLOW_DOWN: f32 = 0.5;
    const STOP: f32 = 0.1;
    pub fn on_event(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => match (input.virtual_keycode, input.state) {
                (Some(VirtualKeyCode::J), ElementState::Released) => {
                    self.offset_velocity.x = -Self::BOUNCE
                }
                (Some(VirtualKeyCode::L), ElementState::Released) => {
                    self.offset_velocity.x = Self::BOUNCE;
                }

                (Some(VirtualKeyCode::K), ElementState::Released) => {
                    self.offset_velocity.z = -Self::BOUNCE
                }
                (Some(VirtualKeyCode::I), ElementState::Released) => {
                    self.offset_velocity.z = Self::BOUNCE
                }

                (Some(VirtualKeyCode::O), ElementState::Released) => {
                    self.offset_velocity.y = -Self::BOUNCE;
                }
                (Some(VirtualKeyCode::U), ElementState::Released) => {
                    self.offset_velocity.y = Self::BOUNCE;
                }

                _ => {}
            },
            _ => {}
        }
    }

    pub fn update(&mut self) {
        let delta = self.last_update.elapsed().as_secs_f32();
        self.last_update = Instant::now();

        self.offset_parameter += self.offset_velocity * delta;

        self.offset_velocity /= 1.0 + (delta * Self::SLOW_DOWN);
        for i in 0..3 {
            if self.offset_velocity[i].abs() < Self::STOP {
                self.offset_velocity[i] = 0.0;
            }
        }
    }

    pub fn new() -> Self {
        OffsetEntity {
            offset_parameter: Vec3::ZERO,
            offset_velocity: Vec3::ZERO,
            last_update: Instant::now(),
        }
    }
}
