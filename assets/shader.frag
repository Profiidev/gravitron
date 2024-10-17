#version 450
#extension GL_EXT_nonuniform_qualifier : require

struct DirectionalLight {
  vec3 direction;
  vec3 color;
  float intensity;
};

struct PointLight {
  vec3 position;
  vec3 color;
  float intensity;
  float range;
};

struct SpotLight {
  vec3 position;
  vec3 direction;
  vec3 color;
  float intensity;
  float range;
  float angle;
};

layout (set=0, binding=1) uniform sampler2D textures[];

layout (set=0, binding=2) uniform LightInfo {
  uint num_point_lights;
  uint num_spot_lights;
  DirectionalLight dl;
} light_info;

layout (set=0, binding=3) buffer readonly PointLights {
  PointLight pls[];
} pls;

layout (set=0, binding=4) buffer readonly SpotLights {
  SpotLight sls[];
} sls;

layout (location = 0) out vec4 fragColor;

layout (location = 0) in vec3 fragColorIn;
layout (location = 1) in vec3 fragNormalIn;
layout (location = 2) in vec2 fragUvIn;
layout (location = 3) in vec4 fragWorldPosIn;
layout (location = 4) in vec3 cameraPosIn;
layout (location = 5) in float metallic;
layout (location = 6) in float roughness;
layout (location = 7) flat in uint textureId;

const float PI = 3.14159265359;

void main() {
  float fac = max(dot(fragNormalIn, light_info.dl.direction), 0.0);
  vec3 color = light_info.dl.color * light_info.dl.intensity * fac;
  fragColor = texture(textures[textureId], fragUvIn) + vec4(fragColorIn, 1.0);
}
