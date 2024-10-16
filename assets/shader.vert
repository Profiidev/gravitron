#version 450

layout (location=0) in vec3 position;
layout (location=1) in vec3 normal;
layout (location=2) in vec2 uv;
layout (location=3) in mat4 model_matrix;
layout (location=7) in mat4 inverse_model_matrix;
layout (location=11) in vec3 colour;
layout (location=12) in float metallic;
layout (location=13) in float roughness;
layout (location=14) in uint texture_id;

layout(set=0, binding=0) uniform UniformBufferObject {
  mat4 view_matrix;
  mat4 projection_matrix;
} ubo;

layout (location=0) out vec3 fragColor;
layout (location=1) out vec3 fragNormal;
layout (location=2) out vec2 fragUv;
layout (location=3) out vec4 fragWorldPos;
layout (location=4) out vec3 cameraPos;
layout (location=5) out float fragMetallic;
layout (location=6) out float fragRoughness;
layout (location=7) out uint fragTextureId;

void main() {
  fragWorldPos = model_matrix * vec4(position,1.0);
  gl_Position = ubo.projection_matrix * ubo.view_matrix * fragWorldPos;
  fragColor = colour;
  fragNormal = transpose(mat3(inverse_model_matrix)) * normal;
  fragUv = uv;
  fragMetallic = metallic;
  fragRoughness = roughness;
  fragTextureId = texture_id;

  cameraPos =
    - ubo.view_matrix[3][0] * vec3(ubo.view_matrix[0][0], ubo.view_matrix[1][0], ubo.view_matrix[2][0])
    - ubo.view_matrix[3][1] * vec3(ubo.view_matrix[0][1], ubo.view_matrix[1][1], ubo.view_matrix[2][1])
    - ubo.view_matrix[3][2] * vec3(ubo.view_matrix[0][2], ubo.view_matrix[1][2], ubo.view_matrix[2][2]);
}