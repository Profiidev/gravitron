#version 450

layout (location = 0) out vec4 fragColor;

layout (location = 0) in vec3 fragColorIn;
layout (location = 1) in vec3 fragNormalIn;
layout (location = 2) in vec4 fragWorldPosIn;

struct DirectionalLight {
  vec3 direction_to_light;
  vec3 irradiance;
};

struct PointLight {
  vec3 position;
  vec3 irradiance;
};

const float PI = 3.14159265359;

vec3 compute_radiance(vec3 irradiance, vec3 light_direction, vec3 normal, vec3 color) {
  return irradiance * max(dot(normal, light_direction), 0) * color;
}

void main() {
  vec3 l = vec3(0);

  DirectionalLight light = DirectionalLight(normalize(vec3(-1.0, 1.0, 0.0)), vec3(10.0, 10.0, 10.0));

  l += compute_radiance(light.irradiance, light.direction_to_light, fragNormalIn, fragColorIn);

  const int NUMBER_OF_POINTLIGHTS = 3;
	
	PointLight pointlights [NUMBER_OF_POINTLIGHTS] = { 
		PointLight(vec3(1.5,0.0,0.0),vec3(10,10,10)),
		PointLight(vec3(1.5,0.2,0.0),vec3(5,5,5)),
		PointLight(vec3(1.6,-0.2,0.1),vec3(5,5,5))
	};

  for(int i = 0; i < NUMBER_OF_POINTLIGHTS; i++){
    PointLight point_light = PointLight(vec3(1.5, 0.0, 0.0), vec3(10.0, 10.0, 10.0));
    vec3 to_light = normalize(point_light.position - fragWorldPosIn.xyz);
    float d = length(fragWorldPosIn.xyz - point_light.position);
    vec3 irradiance = point_light.irradiance / (4 * PI * d * d);

    l += compute_radiance(irradiance, to_light, fragNormalIn, fragColorIn);
  }

  fragColor = vec4(l / (1 + l), 1.0);
}
