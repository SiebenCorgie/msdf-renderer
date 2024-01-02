#![no_std]
#![feature(asm_experimental_arch)]

use core::f32::consts::PI;

use marpii_rmg_shared::ResourceHandle;

pub use spirv_std;
pub use spirv_std::glam;
use spirv_std::glam::{Quat, Vec2, Vec3};

#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

#[cfg_attr(not(target_arch = "spirv"), derive(Clone, Copy, Debug))]
#[cfg_attr(target_arch = "spirv", derive(Clone, Copy))]
#[repr(C, align(16))]
pub struct RenderUniform {
    pub camera_pos: [f32; 3],
    pub fov: f32,
    pub camera_rotation: [f32; 4],
    pub resolution: [u32; 2],
    pub target_image: ResourceHandle,
    pub pad1: [u32; 2],
    pub offset: [f32; 3],
    pub pad2: f32,
}

impl Default for RenderUniform {
    fn default() -> Self {
        RenderUniform {
            camera_pos: [0.0, 0.0, 0.0],
            fov: 90.0,
            camera_rotation: Quat::IDENTITY.into(),
            resolution: [100, 100],
            target_image: ResourceHandle::INVALID,
            pad1: [0; 2],
            offset: [0.0; 3],
            pad2: 0.0,
        }
    }
}

impl RenderUniform {
    fn aspect_ratio(&self) -> f32 {
        self.resolution[0] as f32 / self.resolution[1] as f32
    }
    //ndc in -1.0 .. 1.0
    pub fn ray_from_ndc(&self, ndc: Vec2) -> Ray {
        let px = ndc.x * (self.fov / 2.0 * PI / 180.0).tan() * self.aspect_ratio();
        let py = ndc.y * (self.fov / 2.0 * PI / 180.0).tan();
        let direction = Vec3::new(px, py, 1.0);

        //now rotate for camera
        let direction = Quat::from_array(self.camera_rotation).mul_vec3(direction);

        Ray {
            direction,
            max_t: 50.0,
            origin: self.camera_pos.into(),
        }
    }
}

pub struct Ray {
    pub origin: Vec3,
    pub max_t: f32,
    pub direction: Vec3,
}

impl Ray {
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}
