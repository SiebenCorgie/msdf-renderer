use std::time::Instant;

use shared::glam::{EulerRot, Quat, Vec3};
use winit::event::{DeviceEvent, ElementState, Event, VirtualKeyCode, WindowEvent};

#[derive(Clone, Copy)]
struct MoveState {
    neg: bool,
    pos: bool,
}

impl Default for MoveState {
    fn default() -> Self {
        MoveState {
            neg: false,
            pos: false,
        }
    }
}

impl MoveState {
    fn into_velocity(&self) -> f32 {
        match (self.neg, self.pos) {
            (true, true) => 0.0,
            (false, true) => 1.0,
            (true, false) => -1.0,
            (false, false) => 0.0,
        }
    }
}

pub struct Camera {
    location: Vec3,
    rotation: Quat,
    last_update: Instant,

    speedup: bool,
    move_states: [MoveState; 3],
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            location: Vec3::new(0.0, 0.0, -3.0),
            rotation: Quat::IDENTITY,
            last_update: Instant::now(),
            speedup: false,
            move_states: [MoveState::default(); 3],
        }
    }
}

impl Camera {
    const CAM_SPEEDUP: f32 = 0.001;
    pub fn on_event(&mut self, event: &winit::event::Event<()>) {
        match event {
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => match (input.virtual_keycode, input.state) {
                (Some(VirtualKeyCode::A), ElementState::Pressed) => self.move_states[0].neg = true,
                (Some(VirtualKeyCode::A), ElementState::Released) => {
                    self.move_states[0].neg = false
                }
                (Some(VirtualKeyCode::D), ElementState::Pressed) => self.move_states[0].pos = true,
                (Some(VirtualKeyCode::D), ElementState::Released) => {
                    self.move_states[0].pos = false
                }

                (Some(VirtualKeyCode::S), ElementState::Pressed) => self.move_states[2].neg = true,
                (Some(VirtualKeyCode::S), ElementState::Released) => {
                    self.move_states[2].neg = false
                }
                (Some(VirtualKeyCode::W), ElementState::Pressed) => self.move_states[2].pos = true,
                (Some(VirtualKeyCode::W), ElementState::Released) => {
                    self.move_states[2].pos = false
                }

                (Some(VirtualKeyCode::E), ElementState::Pressed) => self.move_states[1].neg = true,
                (Some(VirtualKeyCode::E), ElementState::Released) => {
                    self.move_states[1].neg = false
                }
                (Some(VirtualKeyCode::Q), ElementState::Pressed) => self.move_states[1].pos = true,
                (Some(VirtualKeyCode::Q), ElementState::Released) => {
                    self.move_states[1].pos = false
                }

                (Some(VirtualKeyCode::LShift), ElementState::Pressed) => self.speedup = true,
                (Some(VirtualKeyCode::LShift), ElementState::Released) => self.speedup = false,

                _ => {}
            },
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                let right = self.rotation.mul_vec3(Vec3::new(1.0, 0.0, 0.0));
                let rot_yaw = Quat::from_rotation_y(delta.0 as f32 * Self::CAM_SPEEDUP);
                let rot_pitch = Quat::from_axis_angle(right, delta.1 as f32 * Self::CAM_SPEEDUP);

                let to_add = rot_yaw * rot_pitch;
                self.rotation = to_add * self.rotation;
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        let delta = self.last_update.elapsed().as_secs_f32();
        self.last_update = Instant::now();

        let mut velocity = Vec3::new(
            self.move_states[0].into_velocity(),
            self.move_states[1].into_velocity(),
            self.move_states[2].into_velocity(),
        );

        if self.speedup {
            velocity *= 10.0;
        }
        let velo_div = self.rotation.mul_vec3(velocity);

        self.location += velo_div * delta;
    }

    pub fn get_gpu_dta(&self) -> ([f32; 3], [f32; 4]) {
        //println!("{} @ {}", self.location, self.rotation.mul_vec3(Vec3::Z));
        (self.location.into(), self.rotation.to_array())
    }
}
