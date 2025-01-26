#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec2 inUv;
layout (location = 0) out vec4 outFragColor;

layout (set=1, binding=0) uniform sampler2D actor_sampler;

void main() {
    vec2 uv = inUv / vec2(textureSize(actor_sampler, 0));
    outFragColor = texture(actor_sampler, uv);
}
