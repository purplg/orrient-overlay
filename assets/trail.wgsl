// The time since startup data is in the globals binding which is part of the mesh_view_bindings import
#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::VertexOutput,
}

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
@group(2) @binding(1) var material_color_texture: texture_2d<f32>;
@group(2) @binding(2) var material_color_sampler: sampler;
@group(2) @binding(3) var<uniform> speed: f32;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
  return material_color * textureSample(material_color_texture,
                                        material_color_sampler,
                                        vec2(mesh.uv.x, mesh.uv.y + globals.time * speed));
}
