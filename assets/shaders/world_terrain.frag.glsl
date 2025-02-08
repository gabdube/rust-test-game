#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec2 inPos;
layout (location = 0) out vec4 outFragColor;

layout (set=0, binding=0) uniform sampler2D terrain_sampler;

void main() {
    outFragColor = vec4(0.0, 0.0, 0.0, 1.0);
}
