use marpii::{
    ash::vk::{self, Extent2D},
    resources::{ComputePipeline, ImgDesc, PipelineLayout, PushConstant, ShaderStage},
    OoS,
};
use marpii_rmg::{CtxRmg, ImageHandle, Rmg, Task};

use crate::{patcher::Patcher, Camera};
use shared::{glam::Vec3, RenderUniform};
use std::sync::Arc;

pub struct SphereTracing {
    patcher: Patcher,
    pipeline_layout: Arc<PipelineLayout>,
    pipeline: Arc<ComputePipeline>,
    pub target_image: ImageHandle,
    pc: PushConstant<shared::RenderUniform>,
}

impl SphereTracing {
    pub fn new(rmg: &mut Rmg, base_resolution: Extent2D) -> Self {
        println!("Create for resolution: {base_resolution:?}");
        let mut patcher = Patcher::new(rmg.ctx.device.clone());
        let base_shader = patcher
            .fetch_new_module()
            .expect("Could not get base shader!");

        let pc = PushConstant::new(RenderUniform::default(), vk::ShaderStageFlags::COMPUTE);
        let shader_stage = ShaderStage::from_module(
            base_shader,
            vk::ShaderStageFlags::COMPUTE,
            "renderer".to_owned(),
        );
        let pipeline = Arc::new(
            ComputePipeline::new(
                &rmg.ctx.device,
                &shader_stage,
                None,
                rmg.resources.bindless_layout(),
            )
            .unwrap(),
        );

        let target_image = rmg
            .new_image_uninitialized(
                ImgDesc::storage_image_2d(
                    base_resolution.width,
                    base_resolution.height,
                    vk::Format::R32G32B32A32_SFLOAT,
                ),
                Some("st_pass_target"),
            )
            .unwrap();

        SphereTracing {
            target_image,
            patcher,
            pc,
            pipeline_layout: rmg.resources.bindless_layout(),
            pipeline,
        }
    }

    pub fn try_new_pipeline(&mut self, ctx: &CtxRmg) {
        let nm = if let Some(new_module) = self.patcher.fetch_new_module() {
            log::info!("New Shader module!");
            new_module
        } else {
            return;
        };

        let ss = ShaderStage::from_module(nm, vk::ShaderStageFlags::COMPUTE, "renderer".to_owned());

        match ComputePipeline::new(&ctx.device, &ss, None, self.pipeline_layout.clone()) {
            Ok(np) => {
                self.pipeline = Arc::new(np);
            }
            Err(e) => {
                log::error!("Failed to build new pipeline: {e}");
                return;
            }
        }
    }

    pub fn offset_parameter(&mut self, new: Vec3) {
        self.pc.get_content_mut().offset = new.into();
    }

    pub fn notify_resolution(&mut self, rmg: &mut Rmg, resolution: Extent2D) {
        if self.target_image.extent_2d() == resolution {
            return;
        }

        log::info!("Changing resolution to {:?}", resolution);

        let mut desc = self.target_image.image_desc().clone();
        desc.extent.width = resolution.width;
        desc.extent.height = resolution.height;
        self.target_image = rmg
            .new_image_uninitialized(desc, Some("st_pass_resolution"))
            .unwrap();
    }

    pub fn dispatch_size(&self) -> [u32; 3] {
        [
            (self.target_image.extent_2d().width / 8) + 1,
            (self.target_image.extent_2d().height / 8) + 1,
            1,
        ]
    }

    pub fn update_camera(&mut self, cam: &Camera) {
        let (origin, rotation) = cam.get_gpu_dta();
        self.pc.get_content_mut().camera_rotation = rotation;
        self.pc.get_content_mut().camera_pos = origin.into();
    }
}

impl Task for SphereTracing {
    fn name(&self) -> &'static str {
        "Sphere tracing"
    }
    fn queue_flags(&self) -> vk::QueueFlags {
        vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE
    }
    fn register(&self, registry: &mut marpii_rmg::ResourceRegistry) {
        registry
            .request_image(
                &self.target_image,
                vk::PipelineStageFlags2::COMPUTE_SHADER,
                vk::AccessFlags2::SHADER_STORAGE_WRITE,
                vk::ImageLayout::GENERAL,
            )
            .unwrap();

        registry.register_asset(self.pipeline.clone());
    }

    fn pre_record(
        &mut self,
        resources: &mut marpii_rmg::Resources,
        ctx: &marpii_rmg::CtxRmg,
    ) -> Result<(), marpii_rmg::RecordError> {
        self.try_new_pipeline(ctx);

        self.pc.get_content_mut().resolution = [
            self.target_image.extent_2d().width,
            self.target_image.extent_2d().height,
        ];
        self.pc.get_content_mut().target_image = resources
            .resource_handle_or_bind(self.target_image.clone())
            .unwrap();
        Ok(())
    }

    fn record(
        &mut self,
        device: &std::sync::Arc<marpii::context::Device>,
        command_buffer: &vk::CommandBuffer,
        _resources: &marpii_rmg::Resources,
    ) {
        unsafe {
            device.inner.cmd_bind_pipeline(
                *command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.pipeline.pipeline,
            );
            device.inner.cmd_push_constants(
                *command_buffer,
                self.pipeline.layout.layout,
                vk::ShaderStageFlags::ALL,
                0,
                self.pc.content_as_bytes(),
            );

            let [dx, dy, dz] = self.dispatch_size();
            device.inner.cmd_dispatch(*command_buffer, dx, dy, dz);
        }
    }
}
