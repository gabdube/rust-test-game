#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec2 inUv;
layout (location = 0) out vec4 outFragColor;

layout (set=0, binding=1) uniform sampler2D terrain_sampler;

void main() {
    vec2 uv = inUv / vec2(textureSize(terrain_sampler, 0));
    outFragColor = texture(terrain_sampler, uv);
}
