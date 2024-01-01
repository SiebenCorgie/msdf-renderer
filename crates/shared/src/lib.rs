#![no_std]
#![feature(asm_experimental_arch)]

use marpii_rmg_shared::ResourceHandle;

pub use spirv_std;
pub use spirv_std::glam;

#[cfg_attr(not(target_arch = "spirv"), derive(Clone, Copy, Debug))]
#[cfg_attr(target_arch = "spirv", derive(Clone, Copy))]
#[repr(C, align(16))]
pub struct RenderUniform {
    pub camera_pos: [f32; 3],
    pub fov: f32,
    pub camera_direction: [f32; 3],
    pub pad0: f32,
    pub resolution: [u32; 2],
    pub target_image: ResourceHandle,
    pub pad1: [u32; 2],
}

impl Default for RenderUniform {
    fn default() -> Self {
        RenderUniform {
            camera_pos: [0.0, 0.0, 0.0],
            fov: 90.0,
            camera_direction: [0.0, 0.0, 1.0],
            pad0: 0.0,
            resolution: [100, 100],
            target_image: ResourceHandle::INVALID,
            pad1: [0; 2],
        }
    }
}
