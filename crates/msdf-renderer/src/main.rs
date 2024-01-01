//! A really simple sphere trace renderer.
//!
//! uses the `sdf.minisdf` file to runtime-patch the sphere-tracing shader with new code.
//!
//! This is a test intersection of two projects. The [minisdf]() compiler, and the [spv-patcher]().

use std::time::Instant;

use camera::Camera;
use marpii::{ash::vk::Extent2D, context::Ctx};
use marpii_rmg::Rmg;
use marpii_rmg_tasks::SwapchainPresent;
use shared::glam::{EulerRot, Quat, Vec3};
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

mod camera;
mod patcher;
mod st_pass;

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .init()
        .unwrap();

    let ev = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&ev).unwrap();
    let (context, surface) = Ctx::default_with_surface(&window, true).unwrap();
    let mut rmg = Rmg::new(context).unwrap();

    let mut present_pass = SwapchainPresent::new(&mut rmg, surface).unwrap();
    let mut st_pass = st_pass::SphereTracing::new(
        &mut rmg,
        present_pass.extent().unwrap_or(Extent2D {
            width: 1,
            height: 1,
        }),
    );

    let mut camera = Camera::default();

    ev.run(move |ev, _, cf| {
        *cf = ControlFlow::Poll;

        camera.on_event(&ev);

        match ev {
            Event::RedrawRequested(_wid) => {
                camera.update();

                st_pass.update_camera(&camera);

                let resolution = window.inner_size();
                st_pass.notify_resolution(
                    &mut rmg,
                    Extent2D {
                        width: resolution.width,
                        height: resolution.height,
                    },
                );

                present_pass.push_image(
                    st_pass.target_image.clone(),
                    st_pass.target_image.extent_2d(),
                );

                rmg.record()
                    .add_task(&mut st_pass)
                    .unwrap()
                    .add_task(&mut present_pass)
                    .unwrap()
                    .execute()
                    .unwrap();
            }
            Event::LoopDestroyed
            | Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::WindowEvent {
                window_id: _,
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Released,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
            } => *cf = ControlFlow::Exit,
            Event::RedrawEventsCleared => window.request_redraw(),
            _ => {}
        }
    });
}
