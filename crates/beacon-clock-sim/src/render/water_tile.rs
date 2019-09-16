use crate::app::State;
use crate::render::Render;
use crate::render::TextureUnit;
use crate::shader::Shader;
use crate::shader::ShaderKind;
use nalgebra;
use nalgebra::{Isometry3, Matrix4, Vector3};
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;

pub struct RenderableWaterTile<'a> {
    shader: &'a Shader,
}

impl<'a> RenderableWaterTile<'a> {
    pub fn new(shader: &'a Shader) -> RenderableWaterTile<'a> {
        RenderableWaterTile { shader }
    }
}

impl<'a> Render<'a> for RenderableWaterTile<'a> {
    fn shader_kind() -> ShaderKind {
        ShaderKind::Water
    }

    fn shader(&'a self) -> &'a Shader {
        &self.shader
    }

    fn buffer_attributes(&self, gl: &WebGlRenderingContext) {
        let shader = self.shader();

        let pos_attrib = gl.get_attrib_location(&shader.program, "position");
        gl.enable_vertex_attrib_array(pos_attrib as u32);

        // These vertices are the x and z values that create a flat square tile on the `y = 0`
        // plane. In our render function we'll scale this quad into the water size that we want.
        // x and z values, y is omitted since this is a flat surface. We set it in the vertex shader
        let vertices: [f32; 8] = [
            -0.5, 0.5, // Bottom Left
            0.5, 0.5, // Bottom Right
            0.5, -0.5, // Top Right
            -0.5, -0.5, // Top Left
        ];

        let mut indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        RenderableWaterTile::buffer_f32_data(&gl, &vertices, pos_attrib as u32, 2);
        RenderableWaterTile::buffer_u16_indices(&gl, &mut indices);
    }

    fn render(&self, gl: &WebGlRenderingContext, state: &State) {
        let shader = self.shader();

        let model_uni = shader.get_uniform_location(gl, "model");
        let view_uni = shader.get_uniform_location(gl, "view");
        let refraction_texture_uni = shader.get_uniform_location(gl, "refractionTexture");
        let reflection_texture_uni = shader.get_uniform_location(gl, "reflectionTexture");
        let dudv_texture_uni = shader.get_uniform_location(gl, "dudvTexture");
        let normal_map_uni = shader.get_uniform_location(gl, "normalMap");
        let water_depth_texture_uni = shader.get_uniform_location(gl, "waterDepthTexture");
        let dudv_offset_uni = shader.get_uniform_location(gl, "dudvOffset");
        let camera_pos_uni = shader.get_uniform_location(gl, "cameraPos");
        let perspective_uni = shader.get_uniform_location(gl, "perspective");
        let water_reflectivity_uni = shader.get_uniform_location(gl, "waterReflectivity");
        let fresnel_strength_unit = shader.get_uniform_location(gl, "fresnelStrength");

        let pos = (0., 0.0, 0.);

        let x_scale = 18.;
        let z_scale = 18.;
        let scale = Matrix4::new_nonuniform_scaling(&Vector3::new(x_scale, 1.0, z_scale));

        let model = Isometry3::new(Vector3::new(pos.0, pos.1, pos.2), nalgebra::zero());
        let model = model.to_homogeneous();
        let model = scale * model;
        let mut model_array = [0.; 16];
        model_array.copy_from_slice(model.as_slice());
        gl.uniform_matrix4fv_with_f32_array(model_uni.as_ref(), false, &mut model_array);

        let mut view = state.camera().view();
        gl.uniform_matrix4fv_with_f32_array(view_uni.as_ref(), false, &mut view);

        gl.uniform1i(
            refraction_texture_uni.as_ref(),
            TextureUnit::Refraction.texture_unit(),
        );
        gl.uniform1i(
            reflection_texture_uni.as_ref(),
            TextureUnit::Reflection.texture_unit(),
        );
        gl.uniform1i(dudv_texture_uni.as_ref(), TextureUnit::Dudv.texture_unit());
        gl.uniform1i(
            normal_map_uni.as_ref(),
            TextureUnit::NormalMap.texture_unit(),
        );
        gl.uniform1i(
            water_depth_texture_uni.as_ref(),
            TextureUnit::RefractionDepth.texture_unit(),
        );

        gl.uniform1f(water_reflectivity_uni.as_ref(), state.water().reflectivity);

        gl.uniform1f(
            fresnel_strength_unit.as_ref(),
            state.water().fresnel_strength,
        );

        let seconds_elapsed = state.clock() / 1000.;
        let dudv_offset = (state.water().wave_speed * seconds_elapsed) % 1.;
        gl.uniform1f(dudv_offset_uni.as_ref(), dudv_offset);

        let camera_pos = state.camera().get_eye_pos();
        let mut camera_pos = [camera_pos.x, camera_pos.y, camera_pos.z];
        gl.uniform3fv_with_f32_array(camera_pos_uni.as_ref(), &mut camera_pos);

        let mut perspective = state.camera().projection();
        gl.uniform_matrix4fv_with_f32_array(perspective_uni.as_ref(), false, &mut perspective);

        gl.enable(GL::BLEND);
        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);

        gl.draw_elements_with_i32(GL::TRIANGLES, 6, GL::UNSIGNED_SHORT, 0);

        gl.disable(GL::BLEND);
    }
}
