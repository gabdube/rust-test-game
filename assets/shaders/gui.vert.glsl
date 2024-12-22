#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec2 inPos;
layout (location = 1) in vec2 inUv;

layout (location = 0) out vec2 outUv;

layout (push_constant) uniform ScreenInfo {
    layout(offset=0) float screen_width;
    layout(offset=4) float screen_height;
};

void main() {
    outUv = inUv;
    gl_Position = vec4((inPos / vec2(screen_width, screen_height) * 2.0) - vec2(1.0), 0.0, 1.0);
}
