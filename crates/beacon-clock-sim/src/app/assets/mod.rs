use bincode;
use blender_armature::BlenderArmature;
use blender_mesh::BlenderMesh;
use std::collections::HashMap;

#[derive(Default)]
pub struct Assets {
    meshes: HashMap<String, BlenderMesh>,
    armatures: HashMap<String, BlenderArmature>,
}

impl Assets {
    pub fn new() -> Assets {
        let meshes = Assets::download_meshes();
        let armatures = Assets::download_armatures();

        Assets { meshes, armatures }
    }

    // In a real application you would download via XHR or fetch request, but here we just
    // included_bytes! for simplicity
    fn download_meshes() -> HashMap<String, BlenderMesh> {
        let meshes = include_bytes!("../../../meshes.bytes");
        let mut meshes: HashMap<String, BlenderMesh> = bincode::deserialize(meshes).unwrap();

        for (mesh_name, mesh) in meshes.iter_mut() {
            web_sys::console::log_1(&mesh_name.to_string().into());

            mesh.combine_vertex_indices();
            mesh.triangulate();

            if let Some(_armature_name) = mesh.armature_name.as_ref() {
                mesh.set_groups_per_vertex(4);
            } else {
                mesh.y_up();
            }
        }

        meshes
    }

    // In a real application you would download via XHR or fetch request, but here we just
    // included_bytes! for simplicity
    fn download_armatures() -> HashMap<String, BlenderArmature> {
        let armatures = include_bytes!("../../../armatures.bytes");
        let mut armatures: HashMap<String, BlenderArmature> =
            bincode::deserialize(armatures).unwrap();

        for (armature_name, armature) in armatures.iter_mut() {
            web_sys::console::log_1(&armature_name.to_string().into());

            armature.apply_inverse_bind_poses();
            armature.transpose_actions();
            armature.actions_to_dual_quats();
        }

        armatures
    }

    pub fn get_mesh(&self, mesh_name: &str) -> Option<&BlenderMesh> {
        self.meshes.get(mesh_name)
    }

    pub fn get_armature(&self, armature_name: &str) -> Option<&BlenderArmature> {
        self.armatures.get(armature_name)
    }
}
