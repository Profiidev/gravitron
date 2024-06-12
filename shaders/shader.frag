#version 450

layout (location = 0) out vec4 fragColor;

layout (location = 0) in vec4 fragColorIn;
layout (location = 1) in vec3 fragNormalIn;

void main() {
  vec3 direction_to_light = normalize(vec3(-1.0, 1.0, 0.0));
  fragColor = 0.5*(1+max(dot(fragNormalIn, direction_to_light), 0)) * fragColorIn;
}
