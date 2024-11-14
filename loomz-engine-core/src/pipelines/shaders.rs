use loomz_shared::{backend_init_err, CommonError};
use crate::VulkanContext;

pub static SHADER_ENTRY: &[u8] = b"main\0";

#[derive(Default, Copy, Clone)]
pub struct GraphicsShaderModules {
    pub vert: vk::ShaderModule,
    pub frag: vk::ShaderModule,
}

impl GraphicsShaderModules {

    pub fn new(ctx: &VulkanContext, vert_bytes: &[u8], frag_bytes: &[u8]) -> Result<Self, CommonError> {
        let mut vert_src = vec![0u32; vert_bytes.len() / 4];
        let mut frag_src = vec![0u32; frag_bytes.len() / 4];
        unsafe {
            ::std::ptr::copy_nonoverlapping(vert_bytes.as_ptr(), vert_src.as_mut_ptr() as *mut u8, vert_bytes.len());
            ::std::ptr::copy_nonoverlapping(frag_bytes.as_ptr(), frag_src.as_mut_ptr() as *mut u8, frag_bytes.len());
        }

        let device = &ctx.device;
        let vert_info = vk::ShaderModuleCreateInfo {
            code_size: vert_bytes.len() as _,
            p_code: vert_src.as_ptr(),
            ..Default::default()
        };
        let vert = device.create_shader_module(&vert_info)
            .map_err(|err| backend_init_err!("Failed to compile shader module: {:?}", err) )?;
    
        let frag_info = vk::ShaderModuleCreateInfo {
            code_size: frag_bytes.len() as _,
            p_code: frag_src.as_ptr(),
            ..Default::default()
        };
        let frag = device.create_shader_module(&frag_info)
            .map_err(|err| backend_init_err!("Failed to compile shader module: {:?}", err) )?;

        let shaders = GraphicsShaderModules { vert, frag };

        Ok(shaders)
    }

    pub fn destroy(self, ctx: &VulkanContext) {
        ctx.device.destroy_shader_module(self.vert);
        ctx.device.destroy_shader_module(self.frag);
    }

}
