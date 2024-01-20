#![no_std]
#![feature(asm_experimental_arch)]

use shared::glam::{vec3, vec4};
#[cfg(target_arch = "spirv")]
use shared::spirv_std::num_traits::Float;
use shared::spirv_std::{self, Sampler};
use shared::spirv_std::{spirv, Image, RuntimeArray};
use spirv_std::glam::{IVec2, UVec2, UVec3, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};

//ULTRA VIOLET
//const FOG_COLOR: Vec3 = vec3(95.0 / 255.0, 75.0 / 255.0, 139.0 / 255.0);
const FOG_COLOR: Vec3 = vec3(56.0 / 255.0, 52.0 / 255.0, 49.0 / 255.0);

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
pub fn eval_sdf(pos: Vec3, offset: Vec3) -> f32 {
    (pos - offset).length() - 1.0
}

//Uses the thetrahedron technique described here: https://iquilezles.org/articles/normalsSDF/
fn calc_normal(at: Vec3, offset: Vec3) -> Vec3 {
    const H: f32 = 0.00001;
    (vec3(1.0, -1.0, -1.0) * eval_sdf(at + (H * vec3(1.0, -1.0, -1.0)), offset)
        + vec3(-1.0, -1.0, 1.0) * eval_sdf(at + (H * vec3(-1.0, -1.0, 1.0)), offset)
        + vec3(-1.0, 1.0, -1.0) * eval_sdf(at + (H * vec3(-1.0, 1.0, -1.0)), offset)
        + vec3(1.0, 1.0, 1.0) * eval_sdf(at + (H * vec3(1.0, 1.0, 1.0)), offset))
    .normalize()
}
fn fresnel(u: f32, f0: Vec3) -> Vec3 {
    f0 + (Vec3::ONE - f0) * (1.0 - u).powf(5.0)
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

    let ndc = coord_uv * 2.0 - 1.0;
    let ray = push.ray_from_ndc(ndc);

    let mut t = 0.001f32;
    let mut i = 0;
    const EPS: f32 = 0.0001;
    const MAX_I: usize = 1_000_000;

    while t < ray.max_t && i < MAX_I {
        let res = eval_sdf(ray.at(t), Vec3::from(push.offset));
        if res <= EPS {
            break;
        } else {
            t += res;
        }
        i += 1;
    }

    let fog_base = (t / ray.max_t).clamp(0.0, 1.0);
    let fog_color = FOG_COLOR;

    if i >= MAX_I {
        unsafe {
            rgbaf32_images
                .index(push.target_image.index() as usize)
                .write(coord, Vec3::X.extend(1.0));
        }
        return;
    }

    //Early out as _sky_ if we ended the ray
    if t > ray.max_t {
        unsafe {
            rgbaf32_images
                .index(push.target_image.index() as usize)
                .write(coord, reinhard_map(fog_color.extend(1.0)));
        }

        return;
    }

    let nrm = calc_normal(ray.at(t), Vec3::from(push.offset));
    //NOTE: Flipping cause we are in Vulkan space with -Y == UP.
    const LIGHT_DIR: Vec3 = vec3(1.0, -1.0, 1.0);

    /*let base_color = if (((t / ray.max_t) * 10.0) as i32) % 2 == 1 {
        Vec3::new(1.0, 1.0, 1.0)
    } else {
        Vec3::new(1.0, 0.5, 0.5)
    };*/

    let base_color = vec3(228.0 / 255.0, 232.0 / 255.0, 230.0 / 255.0);

    let n_dot_l = LIGHT_DIR.dot(nrm);
    let ao = eval_sdf(ray.at(t) + nrm * 0.2, Vec3::from(push.offset));

    let rim_light = Vec3::splat(1.0 - nrm.dot(-ray.direction)) * base_color * 0.2;
    let direct_light = Vec3::new(1.0, 1.0, 1.0) * n_dot_l.max(0.1);
    let color = base_color * (direct_light + rim_light) * (ao / 0.2);
    let color = color.lerp(FOG_COLOR, fog_base);

    if push.target_image.is_valid() {
        unsafe {
            rgbaf32_images
                .index(push.target_image.index() as usize)
                .write(coord, reinhard_map(color.extend(1.0)));
        }
    }
}
