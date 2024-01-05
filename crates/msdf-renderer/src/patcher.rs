use hotwatch::{notify::Event, Hotwatch};
use marpii::{resources::ShaderModule, OoS};
use patch_function::StaticReplace;
use std::{
    path::Path,
    sync::{
        mpsc::{Receiver, TryRecvError},
        Arc,
    },
    time::{Duration, Instant},
};

static BASE_SHADER: &'static [u8] = include_bytes!("../../resources/base-shader.spv");

///Hotwatch local patcher that first executes msdfc, then patches that into the
/// base shader.
struct LivePatcher {}

///Patcher utility. Observes the `sdf.minisdf` file, and recompiles the shader if needed.
pub struct Patcher {
    ///Receiver of new shader modules
    recv: Receiver<OoS<ShaderModule>>,
    hotwatch: Hotwatch,
}

impl Patcher {
    pub fn new(device: Arc<marpii::context::Device>) -> Self {
        let (send, recv) = std::sync::mpsc::channel();

        //NOTE: PreSend the first shader module, by just loading the base shader without patches.
        let base_shader = ShaderModule::new_from_bytes(&device, BASE_SHADER)
            .expect("Could not build base-shader module!");

        send.send(OoS::new(base_shader))
            .expect("Failed to send initial base shader!");

        let mut watcher = Hotwatch::new_with_custom_delay(Duration::from_millis(500))
            .expect("Could not create file watcher!");

        if !Path::new("sdf.minisdf").exists() {
            panic!("File sdf.minisdf does not exist!");
        }

        //touch the file

        let mut patcher =
            spv_patcher::Module::new(BASE_SHADER.to_vec()).expect("Could not load basecode");

        watcher
            .watch("sdf.minisdf", move |ev: Event| {
                let start = Instant::now();
                if !ev.kind.is_modify() {
                    return;
                }

                //First of all, try to run the compiler
                let modules = match msdfc::compile_file("sdf.minisdf") {
                    Ok(m) => m,
                    Err(e) => {
                        log::error!("Failed to compile sdf.minisdf: {e}");
                        return;
                    }
                };

                //now, try to inject all fields, based on the name using the patcher for our base module
                let mut patch = patcher.patch();
                for (name, module) in modules {
                    println!("Injecting module {name}");

                    //Locally patch the module's memory model
                    let module = {
                        let mmpatcher = match spv_patcher::Module::new(
                            bytemuck::cast_slice(&module).to_vec(),
                        ) {
                            Err(e) => {
                                log::error!("Could not load patch module {name} into patcher: {e}");
                                continue;
                            }
                            Ok(k) => k,
                        };

                        match mmpatcher.patch().patch(spv_patcher::patch::MemoryModel {
                            from: (
                                patch_function::rspirv::spirv::AddressingModel::Logical,
                                patch_function::rspirv::spirv::MemoryModel::GLSL450,
                            ),
                            to: (
                                patch_function::rspirv::spirv::AddressingModel::Logical,
                                patch_function::rspirv::spirv::MemoryModel::Vulkan,
                            ),
                        }) {
                            Err(e) => {
                                log::error!("Failed to mutate memory model for {name}: {e}");
                                continue;
                            }
                            Ok(k) => k.assemble(),
                        }
                    };

                    let static_patch =
                        match StaticReplace::new_from_bytes(&bytemuck::cast_slice(&module), 0) {
                            Err(e) => {
                                log::error!("Failed to build patch for {name}: {e}");
                                return;
                            }
                            Ok(p) => p,
                        };

                    patch = match patch.patch(static_patch) {
                        Ok(p) => p,
                        Err(e) => {
                            log::error!("Failed to patch {name}: {e}");
                            return;
                        }
                    }
                }

                let new_module_code = patch.assemble();
                match ShaderModule::new_from_bytes(&device, bytemuck::cast_slice(&new_module_code))
                {
                    Ok(sm) => {
                        println!(
                            "Successfuly patched in {}ms",
                            start.elapsed().as_secs_f32() * 1000.0
                        );
                        let _ = send.send(sm.into());
                    }
                    Err(e) => {
                        log::error!("Could not build shader module for patched shader: {e}")
                    }
                }
            })
            .expect("Could not schedule sdf-file watcher!");

        Self {
            recv,
            hotwatch: watcher,
        }
    }

    pub fn fetch_new_module(&mut self) -> Option<OoS<ShaderModule>> {
        match self.recv.try_recv() {
            Ok(new) => Some(new),
            Err(TryRecvError::Disconnected) => {
                log::error!("Patch receiver is disconnected, restart the application!");
                None
            }
            _ => None,
        }
    }
}
