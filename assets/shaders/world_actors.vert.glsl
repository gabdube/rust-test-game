#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec2 inPos;
layout (location = 0) out vec2 outUv;

struct SpriteData {
    vec4 position;
    vec4 texture_coordinates;
};

layout (push_constant) uniform ScreenInfo {
    layout(offset=0) float screen_width;
    layout(offset=4) float screen_height;
};

layout (std430, set=0, binding=0) readonly buffer SpriteDataBuffer {
   SpriteData sprites[];
};

void main() {
    SpriteData data = sprites[gl_InstanceIndex];

    vec4 texcoord = data.texture_coordinates;
    outUv = vec2(
        texcoord.x + (inPos.x * texcoord.z),  
        texcoord.y + (inPos.y * texcoord.w)
    );

    vec4 positions = (data.position / vec4(screen_width, screen_height, screen_width, screen_height)) * 2.0;
    float x = (positions.x - 1.0) + (inPos.x * positions.z);
    float y = (positions.y - 1.0) + (inPos.y * positions.w);
    gl_Position = vec4(x, y, 0.0, 1.0);
}
