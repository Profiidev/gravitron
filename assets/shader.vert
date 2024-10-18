#version 450

layout (location=0) in vec3 position;
layout (location=1) in vec3 normal;
layout (location=2) in vec2 uv;
layout (location=3) in mat4 model_matrix;
layout (location=7) in mat4 inverse_model_matrix;
layout (location=11) in vec4 color;
layout (location=12) in float metallic;
layout (location=13) in float roughness;
layout (location=14) in uint texture_id;

layout(set=0, binding=0) uniform UniformBufferObject {
  mat4 view_matrix;
  mat4 projection_matrix;
} ubo;

layout (location=0) out vec4 color_out;
layout (location=1) out vec3 normal_out;
layout (location=2) out vec2 uv_out;
layout (location=3) out vec4 world_pos;
layout (location=4) out vec3 cam_pos;
layout (location=5) out float metallic_out;
layout (location=6) out float roughness_out;
layout (location=7) out uint texture_id_out;

void main() {
  world_pos = model_matrix * vec4(position,1.0);
  gl_Position = ubo.projection_matrix * ubo.view_matrix * world_pos;
  color_out = color;
  normal_out = transpose(mat3(inverse_model_matrix)) * normal;
  uv_out = uv;
  metallic_out = metallic;
  roughness_out = roughness;
  texture_id_out = texture_id;

  cam_pos =
    - ubo.view_matrix[3][0] * vec3(ubo.view_matrix[0][0], ubo.view_matrix[1][0], ubo.view_matrix[2][0])
    - ubo.view_matrix[3][1] * vec3(ubo.view_matrix[0][1], ubo.view_matrix[1][1], ubo.view_matrix[2][1])
    - ubo.view_matrix[3][2] * vec3(ubo.view_matrix[0][2], ubo.view_matrix[1][2], ubo.view_matrix[2][2]);
}