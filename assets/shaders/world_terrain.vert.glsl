#version 450

// This shader works on a 16x16 chunk. Texture uvs is stored in a storage buffer
// Sprite position (within the chunk) is stored in vertex attribute

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

struct SpriteData {
    vec2 texture_coordinates;
};

layout (location = 0) in vec2 inPos;
layout (location = 1) in vec2 inUv;

layout (location = 0) out vec2 outUv;

layout (push_constant) uniform ScreenInfo {
    layout(offset=0)  float screen_width;
    layout(offset=4)  float screen_height;
    layout(offset=8)  float view_x;
    layout(offset=12) float view_y;
    layout(offset=16) float batch_x;
    layout(offset=20) float batch_y;
};
layout (std430, set=0, binding=0) readonly buffer SpriteDataBuffer {
   SpriteData sprites[];
};

void main() {
    float chunk_stride = 16.0;
    float sprite_pixel_size = 64.0;
    float sprite_texel_size = 64.0;
    uint chunk_cell_count = 16*16;
    uint sprite_local_index = uint(float(gl_VertexIndex) * 0.25); // Not sure this is faster than a division
    uint sprite_index = (gl_InstanceIndex * chunk_cell_count) + sprite_local_index;
    SpriteData data = sprites[sprite_index];
    vec2 texcoord = data.texture_coordinates;
    outUv = vec2(
        texcoord.x + (inUv.x * sprite_texel_size),  
        texcoord.y + (inUv.y * sprite_texel_size)
    );
    vec4 positions = (vec4(view_x+batch_x, view_y+batch_y, sprite_pixel_size, sprite_pixel_size) / vec4(screen_width, screen_height, screen_width, screen_height)) * 2.0;
    float x = (positions.x - 1.0) + (inPos.x * positions.z);
    float y = (positions.y - 1.0) + (inPos.y * positions.w);
    gl_Position = vec4(x, y, 0.0, 1.0);
}
