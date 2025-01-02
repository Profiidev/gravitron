#version 450

layout(set=0, binding=0) uniform UniformBufferObject {
  mat4 view_matrix;
  mat4 projection_matrix;
} ubo;

layout (location=0) in vec3 pos_in;
layout (location=1) in vec3 _;
layout (location=2) in vec2 uv_in;

layout (location=0) out vec2 uv_out;
layout (location=1) out vec3 cam_pos;

void main() {
  uv_out = uv_in;
  gl_Position = vec4(pos_in, 1.0f);

  cam_pos =
    - ubo.view_matrix[3][0] * vec3(ubo.view_matrix[0][0], ubo.view_matrix[1][0], ubo.view_matrix[2][0])
    - ubo.view_matrix[3][1] * vec3(ubo.view_matrix[0][1], ubo.view_matrix[1][1], ubo.view_matrix[2][1])
    - ubo.view_matrix[3][2] * vec3(ubo.view_matrix[0][2], ubo.view_matrix[1][2], ubo.view_matrix[2][2]);
}