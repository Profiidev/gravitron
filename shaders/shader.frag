#version 450

layout (location=0) out vec4 theColour;

layout (location=0) in vec2 uv;

layout(set=1, binding=0) uniform sampler2D tex[];


void main(){
	theColour=texture(tex[1], uv);
}