#version 450

layout(location = 0) in vec4 vertexPosition;
layout(location = 1) in float vertexSize;
layout(location = 2) in vec4 vertexColor;

layout(location = 0) out vec4 fragColor;

void main() {
  gl_PointSize = vertexSize;
  gl_Position = vertexPosition;
  fragColor = vertexColor;
}