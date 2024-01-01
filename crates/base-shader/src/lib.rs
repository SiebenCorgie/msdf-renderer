#![no_std]
#![feature(asm_experimental_arch)]

#[cfg(target_arch = "spirv")]
use shared::spirv_std::num_traits::Float;
use shared::spirv_std::{self, Sampler};
use shared::spirv_std::{spirv, Image, RuntimeArray};
use spirv_std::glam::{IVec2, UVec2, UVec3, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};

fn luminance_rec_709(rgb: Vec3) -> f32 {
    rgb.dot(Vec3::new(0.2126, 0.7152, 0.0722))
}

fn reinhard_map(hdr: Vec4) -> Vec4 {
    (hdr.xyz() / (1.0 + luminance_rec_709(hdr.xyz()))).extend(hdr.w)
}

fn reinhard_inverse(ldr: Vec4) -> Vec4 {
    (ldr.xyz() / (1.0 - luminance_rec_709(ldr.xyz()))).extend(ldr.w)
}

//The Sdf function we are patching
#[inline(never)]
pub fn sdf(pos: Vec3, offset: Vec3) -> f32 {
    pos.length() - offset.x
}

#[spirv(compute(threads(8, 8, 1)))]
pub fn renderer(
    #[spirv(push_constant)] push: &shared::RenderUniform,
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 1, binding = 0)] rgbaf32_images: &RuntimeArray<
        Image!(2D, format = rgba32f, sampled = false),
    >,
) {
    let coord = id.xy();
    if coord.x >= push.resolution[0] || coord.y >= push.resolution[1] {
        return;
    }
    let coordf32 = coord.as_vec2();
    let coord_uv = coordf32 / UVec2::new(push.resolution[0], push.resolution[1]).as_vec2();

    if push.target_image.is_valid() {
        unsafe {
            rgbaf32_images
                .index(push.target_image.index() as usize)
                .write(coord, Vec4::ONE);
        }
    }
}
