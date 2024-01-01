use hotwatch::{notify::Event, Hotwatch};
use marpii::{resources::ShaderModule, OoS};
use std::sync::{
    mpsc::{Receiver, TryRecvError},
    Arc,
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

        let mut watcher = Hotwatch::new().expect("Could not create file watcher!");

        watcher.watch("sdf.minisdf", move |ev: Event| {
            if !ev.kind.is_modify() {
                return;
            }

            //TODO build new module
        });

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
