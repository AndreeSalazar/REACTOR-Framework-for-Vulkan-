//! Fábricas de recursos: meshes, texturas y materiales.
//!
//! Estas funciones son la "API corta" del runtime para crear assets vinculados
//! al `VulkanContext` y al allocator que el `Reactor` ya tiene listos.

use super::Reactor;
use crate::core::error::ReactorResult;
use crate::resources::material::Material;
use crate::resources::mesh::Mesh;
use crate::resources::texture::Texture;
use crate::resources::vertex::Vertex;

impl Reactor {
    /// Crea un mesh GPU a partir de vértices e índices.
    pub fn create_mesh(&self, vertices: &[Vertex], indices: &[u32]) -> ReactorResult<Mesh> {
        Mesh::new(&self.context, &self.allocator, vertices, indices)
    }

    /// Carga una textura desde fichero (PNG/JPG/BMP/HDR/…).
    pub fn load_texture(&self, path: &str) -> ReactorResult<Texture> {
        Texture::from_file(&self.context, self.allocator.clone(), path, true)
    }

    /// Carga una textura lineal desde fichero (normalmente mapas de datos PBR).
    pub fn load_texture_linear(&self, path: &str) -> ReactorResult<Texture> {
        Texture::from_file_linear(&self.context, self.allocator.clone(), path, true)
    }

    /// Carga una textura desde bytes embebidos.
    pub fn load_texture_bytes(&self, bytes: &[u8]) -> ReactorResult<Texture> {
        Texture::from_bytes(&self.context, self.allocator.clone(), bytes, true)
    }

    /// Crea una textura sólida 1×1 del color RGBA dado.
    pub fn create_solid_texture(&self, r: u8, g: u8, b: u8, a: u8) -> ReactorResult<Texture> {
        Texture::solid_color(&self.context, self.allocator.clone(), r, g, b, a)
    }

    /// Crea un material sin texturas usando *Dynamic Rendering*.
    pub fn create_material(&self, vert_code: &[u32], frag_code: &[u32]) -> ReactorResult<Material> {
        Material::new_with_msaa(
            &self.context,
            None, // Dynamic Rendering
            vert_code,
            frag_code,
            self.swapchain.extent.width,
            self.swapchain.extent.height,
            self.msaa_samples,
            self.swapchain.format,
            Some(self.depth_format),
        )
    }

    /// Crea un material con textura difusa usando *Dynamic Rendering*.
    pub fn create_textured_material(
        &self,
        vert_code: &[u32],
        frag_code: &[u32],
        texture: &Texture,
    ) -> ReactorResult<Material> {
        Material::with_texture(
            &self.context,
            None, // Dynamic Rendering
            vert_code,
            frag_code,
            self.swapchain.extent.width,
            self.swapchain.extent.height,
            texture,
            self.msaa_samples,
            self.swapchain.format,
            Some(self.depth_format),
        )
    }

    /// Crea un material con IBL (Image-Based Lighting) usando *Dynamic Rendering*.
    ///
    /// El pipeline se construye con un descriptor set layout vacío en set=0
    /// y el layout de IBL en set=1, permitiendo al fragment shader samplear
    /// los cubemaps de irradiance, prefiltered specular y BRDF LUT.
    pub fn create_ibl_material(
        &self,
        vert_code: &[u32],
        frag_code: &[u32],
        ibl_set_layout: ash::vk::DescriptorSetLayout,
    ) -> ReactorResult<Material> {
        use crate::resources::material::MaterialBuilder;

        // Crear un descriptor set layout vacío para set=0 (el material no tiene
        // texturas propias todavía — el color llega por push constants).
        let empty_layout = unsafe {
            self.context
                .device
                .create_descriptor_set_layout(
                    &ash::vk::DescriptorSetLayoutCreateInfo::default(),
                    None,
                )
                .map_err(|e| {
                    crate::core::error::ReactorError::with_source(
                        crate::core::error::ErrorCode::VulkanPipelineCreation,
                        "Failed to create empty descriptor set layout for IBL material",
                        e,
                    )
                })?
        };

        let mut builder = MaterialBuilder::new(vert_code.to_vec(), frag_code.to_vec())
            .msaa(self.msaa_samples)
            .fragment_shading_rate(self.context.supports_fragment_shading_rate())
            .descriptor_layout(empty_layout)    // set = 0 (vacío)
            .descriptor_layout(ibl_set_layout); // set = 1 (IBL textures)

        if let Some(shadow_layout) = self.shadow_descriptor_layout {
            builder = builder.descriptor_layout(shadow_layout); // set = 2 (Sombras)
        }

        let mut mat = builder
            .uses_ibl(true)
            .build(
                &self.context,
                None,
                self.swapchain.extent.width,
                self.swapchain.extent.height,
                self.swapchain.format,
                Some(self.depth_format),
            )?;

        // Almacenar el layout vacío para limpieza posterior.
        mat.descriptor_layout = Some(empty_layout);
        mat.device = Some(self.context.device.clone());

        Ok(mat)
    }

    /// Crea un material PBR con soporte para IBL, mapa de albedo, mapa de normales, metálico y rugosidad.
    pub fn create_pbr_material(
        &self,
        vert_code: &[u32],
        frag_code: &[u32],
        ibl_set_layout: ash::vk::DescriptorSetLayout,
        albedo_texture: &crate::resources::texture::Texture,
        normal_texture: &crate::resources::texture::Texture,
        metallic_texture: &crate::resources::texture::Texture,
        roughness_texture: &crate::resources::texture::Texture,
    ) -> ReactorResult<Material> {
        use crate::resources::material::MaterialBuilder;
        use ash::vk;

        // 1. Crear el descriptor set layout para Set 0 (albedo + normal + metallic + roughness)
        let albedo_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let normal_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let metallic_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(2)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let roughness_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(3)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let bindings = [
            albedo_binding,
            normal_binding,
            metallic_binding,
            roughness_binding,
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

        let descriptor_layout = unsafe {
            self.context
                .device
                .create_descriptor_set_layout(&layout_info, None)
                .map_err(|e| {
                    crate::core::error::ReactorError::with_source(
                        crate::core::error::ErrorCode::VulkanPipelineCreation,
                        "Failed to create descriptor set layout for PBR material",
                        e,
                    )
                })?
        };

        // 2. Crear descriptor pool
        let pool_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(4);

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(std::slice::from_ref(&pool_size))
            .max_sets(1);

        let descriptor_pool = unsafe {
            self.context
                .device
                .create_descriptor_pool(&pool_info, None)
                .map_err(|e| {
                    crate::core::error::ReactorError::with_source(
                        crate::core::error::ErrorCode::VulkanPipelineCreation,
                        "Failed to create descriptor pool for PBR material",
                        e,
                    )
                })?
        };

        // 3. Alloc descriptor set
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(std::slice::from_ref(&descriptor_layout));

        let descriptor_sets = unsafe {
            self.context
                .device
                .allocate_descriptor_sets(&alloc_info)
                .map_err(|e| {
                    crate::core::error::ReactorError::with_source(
                        crate::core::error::ErrorCode::VulkanPipelineCreation,
                        "Failed to allocate descriptor set for PBR material",
                        e,
                    )
                })?
        };
        let descriptor_set = descriptor_sets[0];

        // 4. Update descriptor set
        let albedo_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(albedo_texture.view())
            .sampler(albedo_texture.sampler_handle());

        let normal_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(normal_texture.view())
            .sampler(normal_texture.sampler_handle());

        let metallic_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(metallic_texture.view())
            .sampler(metallic_texture.sampler_handle());

        let roughness_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(roughness_texture.view())
            .sampler(roughness_texture.sampler_handle());

        let write_albedo = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&albedo_info));

        let write_normal = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&normal_info));

        let write_metallic = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(2)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&metallic_info));

        let write_roughness = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(3)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&roughness_info));

        unsafe {
            self.context.device.update_descriptor_sets(
                &[write_albedo, write_normal, write_metallic, write_roughness],
                &[],
            );
        }

        // 5. Build material pipeline
        let mut builder = MaterialBuilder::new(vert_code.to_vec(), frag_code.to_vec())
            .msaa(self.msaa_samples)
            .fragment_shading_rate(self.context.supports_fragment_shading_rate())
            .descriptor_layout(descriptor_layout)  // set = 0
            .descriptor_layout(ibl_set_layout);     // set = 1

        if let Some(shadow_layout) = self.shadow_descriptor_layout {
            builder = builder.descriptor_layout(shadow_layout); // set = 2 (Sombras)
        }

        let mut mat = builder
            .uses_ibl(true)
            .build(
                &self.context,
                None,
                self.swapchain.extent.width,
                self.swapchain.extent.height,
                self.swapchain.format,
                Some(self.depth_format),
            )?;

        mat.descriptor_set = Some(descriptor_set);
        mat.descriptor_pool = Some(descriptor_pool);
        mat.descriptor_layout = Some(descriptor_layout);
        mat.device = Some(self.context.device.clone());

        Ok(mat)
    }
}
