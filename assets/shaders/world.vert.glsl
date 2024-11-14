#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec2 inPos;
layout (location = 1) in vec4 inColor;

layout (location = 0) out vec4 outColor;

layout (push_constant) uniform ScreenInfo {
    layout(offset=0) float screen_width;
    layout(offset=4) float screen_height;
};

void main() {
    outColor = inColor;

    float x = 2.0 * inPos.x / screen_width - 1.0;
    float y = (1.0 - 2.0 * inPos.y / screen_height) * -1.0;
    gl_Position = vec4(x, y, 0.0, 1.0);
}
