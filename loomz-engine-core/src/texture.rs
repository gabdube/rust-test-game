use loomz_shared::{backend_err, CommonError, assets::AssetsTextureData};
use crate::staging::StagingImageCopy;
use super::LoomzEngineCore;

#[derive(Copy, Clone)]
pub struct Texture {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory_offset: vk::DeviceSize,
    pub format: vk::Format,
    pub extent: vk::Extent3D,
}

impl LoomzEngineCore {

    pub fn create_texture_from_asset(&mut self, asset: &AssetsTextureData) -> Result<Texture, CommonError> {
        let asset_data = &asset.data;
        let format = asset_data.format();
        let extent = asset_data.extent();

        let mut texture = Texture {
            image: vk::Image::null(),
            view: vk::ImageView::null(),
            memory_offset: vk::DeviceSize::MAX,
            format,
            extent,
        };

        self.create_image(&mut texture)
            .map_err(|err| backend_err!("Failed to create image: {err}") )?;
    
        self.allocate_image_memory(&mut texture)
            .map_err(|err| backend_err!("Failed to allocate image memory: {err}") )?;

        self.upload_image_memory(asset, &mut texture);

        self.create_base_view(&mut texture)
            .map_err(|err| backend_err!("Failed to create image view: {err}") )?;

        Ok(texture)
    }

    pub fn destroy_texture(&mut self, texture: Texture) {
        let device = &self.ctx.device;
        device.destroy_image_view(texture.view);
        device.destroy_image(texture.image);
    }

    fn create_image(&mut self, texture: &mut Texture) -> Result<(), vk::VkResult> {
        let image_create_info = vk::ImageCreateInfo {
            format: texture.format,
            extent: texture.extent,
            image_type: vk::ImageType::TYPE_2D,
            usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            ..Default::default()
        };
        texture.image = self.ctx.device.create_image(&image_create_info)?;
        Ok(())
    }
    
    fn allocate_image_memory(&mut self, texture: &mut Texture) -> Result<(), vk::VkResult> {
        let device = &self.ctx.device;
        let memory = &mut self.resources.images_alloc;
        let image_memory_req = device.get_image_memory_requirements(texture.image);
        let memory_offset = memory.allocate_memory(&image_memory_req);
        device.bind_image_memory(texture.image, memory.handle, memory_offset)?;

        texture.memory_offset = memory_offset;

        Ok(())
    }

    /**
        TODO:

        * More finely grained layers and subresource range
        * Texture with more than one mipmap level
        * Texture usage other than SHADER_READ_ONLY_OPTIMAL
    */
    fn upload_image_memory(&mut self, asset: &AssetsTextureData, texture: &mut Texture) {
        let staging = &mut self.staging;

        // 
        let image_subresource = vk::ImageSubresourceLayers::base_color();
        let subresource_range = vk::ImageSubresourceRange::base_color();

        let pixel_data = asset.data.mimap_level_data(0); // TODO, 
        let buffer_offset = staging.copy_data(pixel_data);

        // Pixel copy
        let image_copy = StagingImageCopy {
            dst_image: texture.image,
            copy: vk::BufferImageCopy {
                buffer_offset,
                buffer_image_height: 0,
                buffer_row_length: 0,
                image_subresource,
                image_offset: vk::Offset3D::default(),
                image_extent: texture.extent,
            }
        };
        staging.image_copies.push(image_copy);

        // Transfer prepare
        let mut barrier = vk::ImageMemoryBarrier2 {
            image: texture.image,
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            src_access_mask: vk::AccessFlags2::NONE,
            src_stage_mask: vk::PipelineStageFlags2::NONE,
            dst_access_mask: vk::AccessFlags2::TRANSFER_WRITE,
            dst_stage_mask: vk::PipelineStageFlags2::ALL_TRANSFER,
            subresource_range,
            ..Default::default()
        };
        staging.image_barrier_prepare.push(barrier);

        // Transfer finalize
        // TODO: Support
        barrier = vk::ImageMemoryBarrier2 {
            image: texture.image,
            old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            src_access_mask: vk::AccessFlags2::TRANSFER_WRITE,
            src_stage_mask: vk::PipelineStageFlags2::ALL_TRANSFER,
            dst_access_mask: vk::AccessFlags2::SHADER_READ,
            dst_stage_mask: vk::PipelineStageFlags2::FRAGMENT_SHADER,
            subresource_range,
            ..Default::default()
        };
        staging.image_barrier_final.push(barrier);
    }

    fn create_base_view(&mut self, texture: &mut Texture) -> Result<(), vk::VkResult> {
        let view_info = vk::ImageViewCreateInfo {
            image: texture.image,
            format: texture.format,
            view_type: vk::ImageViewType::TYPE_2D,
            subresource_range: vk::ImageSubresourceRange::base_color(),
            ..Default::default()
        };

        texture.view = self.ctx.device.create_image_view(&view_info)?;

        Ok(())
    }

}
