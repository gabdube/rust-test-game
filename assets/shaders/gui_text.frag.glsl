#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec2 inUv;
layout (location = 1) in vec4 inColor;

layout (location = 0) out vec4 outFragColor;

layout (set=0, binding=0) uniform sampler2D color;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    vec2 uv = inUv / vec2(textureSize(color, 0));
    vec3 msdf = texture(color, uv).rgb;
    float dist = median(msdf.r, msdf.g, msdf.b);

    float dx = dFdx(uv.x);
    float dy = dFdy(uv.y);
    float toPixels = 8.0 * inversesqrt(dx * dx + dy * dy);
    float w = fwidth(dist) / 1.5;
    float opacity = smoothstep(0.5 - w, 0.5 + w, dist);

    outFragColor = vec4(1.0, 1.0, 1.0, opacity) * vec4(inColor.rgb, 1.0);
}