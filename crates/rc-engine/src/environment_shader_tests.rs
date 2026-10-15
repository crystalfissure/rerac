//! Compile the actual modded WGSL through Bevy's shader composer without a GPU or game data.
//!
//! Only the unchanged Bevy view/mesh imports are substituted with typed, deterministic inputs;
//! our lighting helpers, storage layouts, entry points and alpha/blend variants are the real files.

use bevy::asset::Assets;
use bevy::render::render_resource::{DownlevelFlags, WgpuFeatures};
use bevy::shader::{Shader, ShaderCache, ShaderCacheError, ShaderCacheSource, ShaderDefVal};

const VIEW_TRANSFORMS: &str = r#"
#define_import_path bevy_pbr::view_transformations
fn position_world_to_clip(p: vec3<f32>) -> vec4<f32> { return vec4<f32>(p, 1.0); }
fn position_world_to_view(p: vec3<f32>) -> vec3<f32> { return p; }
"#;

const MESH_FUNCTIONS: &str = r#"
#define_import_path bevy_pbr::mesh_functions
fn get_tag(i: u32) -> u32 { return i; }
"#;

const VIEW_BINDINGS: &str = r#"
#define_import_path bevy_pbr::mesh_view_bindings
struct View { world_position: vec3<f32>, }
@group(0) @binding(0) var<uniform> view: View;
"#;

#[test]
fn environment_world_shaders_compile_all_alpha_and_blend_variants() {
    // Debug tests use Naga Oil's validating composer, the same preprocessor and WGSL validator
    // Bevy uses before uploading a pipeline. No graphics device is required for this step.
    let mut cache = ShaderCache::new((), WgpuFeatures::empty(), DownlevelFlags::all(), |_, source, _| {
        let ShaderCacheSource::Naga(module) = source else { panic!("expected validated WGSL") };
        let (_, fog) = module.global_variables.iter()
            .find(|(_, v)| v.binding.as_ref().is_some_and(|b| b.group == 2 && b.binding == 2))
            .expect("shared fog/lighting binding");
        Ok(module.types[fog.ty].inner.size(module.to_ctx()))
    });
    let mut shaders = Assets::<Shader>::default();
    let mut handles = Vec::new();
    for (path, source) in [
        ("view_transformations.wgsl", VIEW_TRANSFORMS),
        ("mesh_functions.wgsl", MESH_FUNCTIONS),
        ("mesh_view_bindings.wgsl", VIEW_BINDINGS),
        ("world_lights.wgsl", include_str!("../assets/shaders/world_lights.wgsl")),
        ("display_blend.wgsl", include_str!("../assets/shaders/display_blend.wgsl")),
    ] {
        let shader = Shader::from_wgsl(source, path);
        let handle = shaders.add(shader.clone());
        cache.set_shader(handle.id(), shader);
        handles.push(handle);
    }

    for (path, source, effect_modes) in [
        ("tfrag.wgsl", include_str!("../assets/shaders/tfrag.wgsl"), 1),
        ("tie.wgsl", include_str!("../assets/shaders/tie.wgsl"), 1),
        ("shrub.wgsl", include_str!("../assets/shaders/shrub.wgsl"), 1),
        ("shrub_billboard.wgsl", include_str!("../assets/shaders/shrub_billboard.wgsl"), 1),
        ("moby.wgsl", include_str!("../assets/shaders/moby.wgsl"), 3),
        ("moby_metal.wgsl", include_str!("../assets/shaders/moby_metal.wgsl"), 2),
    ] {
        let shader = Shader::from_wgsl(source, path);
        let handle = shaders.add(shader.clone());
        cache.set_shader(handle.id(), shader);
        for alpha_mode in 0..3 {
            for effect_mode in 0..effect_modes {
                let mut defs = vec![ShaderDefVal::UInt("MATERIAL_BIND_GROUP".into(), 2)];
                if alpha_mode != 0 {
                    defs.push(if alpha_mode == 1 { "GS_ATEST_PASS" } else { "GS_ATEST_FAIL" }.into());
                    defs.push(ShaderDefVal::UInt("GS_AREF".into(), 96));
                }
                if effect_mode != 0 {
                    defs.push(if effect_mode == 1 { "DISPLAY_BLEND_MIX" } else { "DISPLAY_BLEND_ADD" }.into());
                }
                match cache.get(0, handle.id(), &defs) {
                    Ok(size) => assert_eq!(*size, if path.starts_with("moby") { 112 } else { 80 }, "{path} shared buffer ABI"),
                    Err(error) => {
                        let detail = match error {
                            ShaderCacheError::ProcessShaderError(e) => e.emit_to_string(&cache.composer),
                            e => e.to_string(),
                        };
                        panic!("{path}, alpha {alpha_mode}, effect {effect_mode}: {detail}");
                    }
                }
            }
        }
        handles.push(handle);
    }
}
