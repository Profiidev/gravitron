#version 450
#extension GL_EXT_nonuniform_qualifier : require

layout (set=1, binding=0) uniform sampler2D textures[];

layout (location = 0) out vec4 color_out;
layout (location = 1) out vec4 normal_out;
layout (location = 2) out vec4 pos_out;

layout (location = 0) in vec4 color_in;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 uv;
layout (location = 3) in vec3 world_pos;
layout (location = 5) in float metallic;
layout (location = 6) in float roughness;
layout (location = 7) flat in uint texture_id;

void main() {
  color_out = texture(textures[texture_id], uv) + color_in;
  normal_out = vec4(normal, metallic);
  pos_out = vec4(world_pos, roughness);
}
