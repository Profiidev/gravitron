#version 450

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

layout (location=0) in vec2 uv_in;
layout (location=1) in vec3 cam_pos;

layout (location=0) out vec4 color_out;

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

layout (input_attachment_index=0, set=1, binding=0) uniform subpassInput color_in;
layout (input_attachment_index=1, set=1, binding=1) uniform subpassInput normal_in;
layout (input_attachment_index=0, set=1, binding=2) uniform subpassInput pos_in;

const float PI = 3.14159265359;

float geometry(vec3 light_direction, vec3 cam_direction, float roughness, vec3 normal) {
  float n_dot_l = abs(dot(normal, light_direction));
  float n_dot_c = abs(dot(normal, cam_direction));
  return 0.5 / max(0.01, mix(2 * n_dot_l * n_dot_c, n_dot_l + n_dot_c, roughness));
}

float distribution(vec3 half_vector, float roughness, vec3 normal) {
  float n_dot_h = dot(half_vector, normal);
  if(n_dot_h > 0) {
    float r = roughness * roughness;
    return r / (PI * pow(1 + n_dot_h * n_dot_h * (r - 1), 2));
  } else {
    return 0.0;
  }
}

vec3 compute_light(vec3 light_color, vec3 light_direction, vec3 color, vec3 cam_direction, vec3 normal, float metallic, float roughness_in) {
  float n_dot_d = max(dot(normal, light_direction), 0);

  vec3 color_surface = light_color * n_dot_d;

  float roughness = roughness_in * roughness_in;
  vec3 f0 = mix(vec3(0.03), color, vec3(metallic));

  vec3 reflected_color = (f0 + (1 - f0) * pow(1 - n_dot_d, 5)) * color_surface;
  vec3 refracted_color = color_surface - reflected_color;
  vec3 refracted_not_absorbed_color = refracted_color * (1 - metallic);

  vec3 half_vec = normalize(0.5 * (cam_direction + light_direction));
  float n_dot_h = max(dot(normal, half_vec), 0);
  vec3 f = f0 + (1 - f0) * pow(1 - n_dot_h, 5);
  vec3 relevant_reflection = reflected_color * f * geometry(light_direction, cam_direction, roughness, normal) * distribution(half_vec, roughness, normal);

  return refracted_not_absorbed_color * color / PI + relevant_reflection;
}

void main() {
  vec4 pos = subpassLoad(pos_in);
  vec4 color = subpassLoad(color_in);
  vec4 normal_t = subpassLoad(normal_in);

  vec3 normal = normal_t.xyz;
  float metallic = normal_t.a;
  vec3 world_pos = pos.xyz;
  float roughness = pos.a;

  vec3 direction_to_cam = normalize(cam_pos - world_pos);

  DirectionalLight dl = light_info.dl;

  vec3 ret = dl.ambient_color * dl.ambient_intensity * color.rgb;

  ret += compute_light(dl.color, -dl.direction, color.rgb, direction_to_cam, normal, metallic, roughness) * dl.intensity;

  for(uint i = 0; i < light_info.num_pls; i++) {
    PointLight pl = pls.pls[i];
    vec3 direction = normalize(pl.position - world_pos.xyz);
    float d = length(world_pos.xyz - pl.position);

    if(d > pl.range) {
      continue;
    }

    vec3 light_color = pl.color / (4 * PI * d * d);

    ret += compute_light(light_color, direction, color.rgb, direction_to_cam, normal, metallic, roughness) * pl.intensity;
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

    ret += compute_light(light_color, direction, color.rgb, direction_to_cam, normal, metallic, roughness) * sl.intensity;
  }

  color_out = vec4(ret / (1 + ret), color.a);
}