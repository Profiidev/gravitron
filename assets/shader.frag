#version 450
#extension GL_EXT_nonuniform_qualifier : require

struct DirectionalLight {
  vec3 direction;
  vec3 color;
  float intensity;
  vec3 ambient_color;
  float ambient_intensity;
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
  uint num_pls;
  uint num_sls;
  DirectionalLight dl;
} light_info;

layout (set=0, binding=3) buffer readonly PointLights {
  PointLight pls[];
} pls;

layout (set=0, binding=4) buffer readonly SpotLights {
  SpotLight sls[];
} sls;

layout (location = 0) out vec4 color_out;

layout (location = 0) in vec4 color_in;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 uv;
layout (location = 3) in vec4 world_pos;
layout (location = 4) in vec3 cam_pos;
layout (location = 5) in float metallic;
layout (location = 6) in float roughness;
layout (location = 7) flat in uint texture_id;

const float PI = 3.14159265359;

float geometry(vec3 light_direction, vec3 cam_direction, float roughness) {
  float n_dot_l = abs(dot(normal, light_direction));
  float n_dot_c = abs(dot(normal, cam_direction));
  return 0.5 / max(0.01, mix(2 * n_dot_l * n_dot_c, n_dot_l + n_dot_c, roughness));
}

float distribution(vec3 half_vector, float roughness) {
  float n_dot_h = dot(half_vector, normal);
  if(n_dot_h > 0) {
    float r = roughness * roughness;
    return r / (PI * pow(1 + n_dot_h * n_dot_h * (r - 1), 2));
  } else {
    return 0.0;
  }
}

vec3 compute_light(vec3 light_color, vec3 light_direction, vec3 color, vec3 cam_direction) {
  float n_dot_d = max(dot(normal, light_direction), 0);

  vec3 color_surface = light_color * n_dot_d;

  float roughness = roughness * roughness;
  vec3 f0 = mix(vec3(0.03), color, vec3(metallic));

  vec3 reflected_color = (f0 + (1 - f0) * pow(1 - n_dot_d, 5)) * color_surface;
  vec3 refracted_color = color_surface - reflected_color;
  vec3 refracted_not_absorbed_color = refracted_color * (1 - metallic);

  vec3 half_vec = normalize(0.5 * (cam_direction + light_direction));
  float n_dot_h = max(dot(normal, half_vec), 0);
  vec3 f = f0 + (1 - f0) * pow(1 - n_dot_h, 5);
  vec3 relevant_reflection = reflected_color * f * geometry(light_direction, cam_direction, roughness) * distribution(half_vec, roughness);

  return refracted_not_absorbed_color * color / PI + relevant_reflection;
}

void main() {
  vec4 color = texture(textures[texture_id], uv) + color_in;

  vec3 direction_to_cam = normalize(cam_pos - world_pos.xyz);

  DirectionalLight dl = light_info.dl;

  vec3 ret = dl.ambient_color * dl.ambient_intensity * color.rgb;

  ret += compute_light(dl.color, -dl.direction, color.rgb, direction_to_cam) * dl.intensity;

  for(uint i = 0; i < light_info.num_pls; i++) {
    PointLight pl = pls.pls[i];
    vec3 direction = normalize(pl.position - world_pos.xyz);
    float d = length(world_pos.xyz - pl.position);

    if(d > pl.range) {
      continue;
    }

    vec3 light_color = pl.color / (4 * PI * d * d);

    ret += compute_light(light_color, direction, color.rgb, direction_to_cam) * pl.intensity;
  }

  for(uint i = 0; i < light_info.num_sls; i++) {
    SpotLight sl = sls.sls[i];
    vec3 direction = normalize(sl.position - world_pos.xyz);
    float d = length(world_pos.xyz - sl.position);

    if(d > sl.range) {
      continue;
    }

    float angle = acos(dot(-direction, sl.direction));
    if(angle > sl.angle) {
      continue;
    }

    vec3 light_color = sl.color / (4 * PI * d * d);

    ret += compute_light(light_color, direction, color.rgb, direction_to_cam) * sl.intensity;
  }

  color_out = vec4(ret / (1 + ret), color.a);
}
