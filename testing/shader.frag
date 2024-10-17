#version 450
#extension GL_EXT_nonuniform_qualifier : require

layout (set=0, binding=1) uniform sampler2D textures[];

layout (set=0, binding=2) uniform StorageBufferObject {
  float num_directional_lights;
  float num_point_lights;
  vec3 data[];
} sbo;

layout (set=1, binding=0) uniform sampler2D tex;

layout (location = 0) out vec4 fragColor;

layout (location = 0) in vec3 fragColorIn;
layout (location = 1) in vec3 fragNormalIn;
layout (location = 2) in vec2 fragUvIn;
layout (location = 3) in vec4 fragWorldPosIn;
layout (location = 4) in vec3 cameraPosIn;
layout (location = 5) in float metallic;
layout (location = 6) in float roughness;
layout (location = 7) flat in uint textureId;

struct DirectionalLight {
  vec3 direction_to_light;
  vec3 irradiance;
};

struct PointLight {
  vec3 position;
  vec3 irradiance;
};

const float PI = 3.14159265359;

void main() {
  fragColor = texture(tex, fragUvIn);
}
