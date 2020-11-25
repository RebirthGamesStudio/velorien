#version 330 core

#include <constants.glsl>

#define LIGHTING_TYPE (LIGHTING_TYPE_TRANSMISSION | LIGHTING_TYPE_REFLECTION)

#define LIGHTING_REFLECTION_KIND LIGHTING_REFLECTION_KIND_SPECULAR

#if (FLUID_MODE == FLUID_MODE_CHEAP)
#define LIGHTING_TRANSPORT_MODE LIGHTING_TRANSPORT_MODE_IMPORTANCE
#elif (FLUID_MODE == FLUID_MODE_SHINY)
#define LIGHTING_TRANSPORT_MODE LIGHTING_TRANSPORT_MODE_RADIANCE
#endif

#define LIGHTING_DISTRIBUTION_SCHEME LIGHTING_DISTRIBUTION_SCHEME_MICROFACET

#define LIGHTING_DISTRIBUTION LIGHTING_DISTRIBUTION_BECKMANN

#include <globals.glsl>
// Note: The sampler uniform is declared here because it differs for MSAA
#include <anti-aliasing.glsl>
#include <srgb.glsl>
#include <cloud.glsl>

//uniform sampler2D src_depth;

in vec2 f_pos;

layout (std140)
uniform u_locals {
    mat4 proj_mat_inv;
    mat4 view_mat_inv;
};

out vec4 tgt_color;

vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

vec3 _illuminate(float max_light, vec3 view_dir, /*vec3 max_light, */vec3 emitted, vec3 reflected) {
    const float NIGHT_EXPOSURE = 10.0;
    const float DUSK_EXPOSURE = 2.0;//0.8;
    const float DAY_EXPOSURE = 1.0;//0.7;

    const float DAY_SATURATION = 1.0;
    const float DUSK_SATURATION = 0.6;
    const float NIGHT_SATURATION = 0.1;

    const float gamma = /*0.5*//*1.*0*/1.0;//1.0;
    /* float light = length(emitted + reflected);
    float color = srgb_to_linear(emitted + reflected);
    float avg_col = (color.r + color.g + color.b) / 3.0;
    return ((color - avg_col) * light + reflected * avg_col) * (emitted + reflected); */
    // float max_intensity = vec3(1.0);
    vec3 color = emitted + reflected;
    float lum = rel_luminance(color);
    // float lum_sky = lum - max_light;

    // vec3 sun_dir = get_sun_dir(time_of_day.x);
    // vec3 moon_dir = get_moon_dir(time_of_day.x);
    // float sky_light = rel_luminance(
    //         get_sun_color(sun_dir) * get_sun_brightness(sun_dir) * SUN_COLOR_FACTOR +
    //         get_moon_color(moon_dir) * get_moon_brightness(moon_dir));
    float sky_light = lum;

    // Tone mapped value.
    // vec3 T = /*color*//*lum*/color;//normalize(color) * lum / (1.0 + lum);
    // float alpha = 0.5;//2.0;
    // float alpha = mix(
    //     mix(
    //         DUSK_EXPOSURE,
    //         NIGHT_EXPOSURE,
    //         max(sun_dir.z, 0)
    //     ),
    //     DAY_EXPOSURE,
    //     max(-sun_dir.z, 0)
    // );
    float alpha = 1.0;//log(1.0 - lum) / lum;
    // vec3 now_light = moon_dir.z < 0 ? moon_dir : sun_dir;
    // float cos_view_light = dot(-now_light, view_dir);
    // alpha *= exp(1.0 - cos_view_light);
    // sky_light *= 1.0 - log(1.0 + view_dir.z);
    float alph = sky_light > 0.0 && max_light > 0.0 ? mix(1.0 / log(/*1.0*//*1.0 + *//*lum_sky + */1.0 + max_light / (0.0 + sky_light)), 1.0, clamp(max_light - sky_light, 0.0, 1.0)) : 1.0;
    alpha = alpha * alph;// min(alph, 1.0);//((max_light > 0.0 && max_light > sky_light /* && sky_light > 0.0*/) ? /*1.0*/1.0 / log(/*1.0*//*1.0 + *//*lum_sky + */1.0 + max_light - (0.0 + sky_light)) : 1.0);
    // alpha = alpha * min(1.0, (max_light == 0.0 ? 1.0 : (1.0 + abs(lum_sky)) / /*(1.0 + max_light)*/max_light));

    vec3 col_adjusted = lum == 0.0 ? vec3(0.0) : color / lum;

    // float L = lum == 0.0 ? 0.0 : log(lum);


    // // float B = T;
    // // float B = L + log(alpha);
    // float B = lum;

    // float D = L - B;

    // float o = 0.0;//log(PERSISTENT_AMBIANCE);
    // float scale = /*-alpha*/-alpha;//1.0;

    // float B_ = (B - o) * scale;

    // // float T = lum;
    // float O = exp(B_ + D);

    float T = 1.0 - exp(-alpha * lum);//lum / (1.0 + lum);
    // float T = lum;

    // Heuristic desaturation
    // const float s = 0.8;
    float s = 1.0;
    // float s = mix(
    //     mix(
    //         DUSK_SATURATION,
    //         NIGHT_SATURATION,
    //         max(sun_dir.z, 0)
    //     ),
    //     DAY_SATURATION,
    //     max(-sun_dir.z, 0)
    // );
    // s = max(s, (max_light) / (1.0 + s));
    // s = max(s, max_light / (1.0 + max_light));
    // s = max_light / (1.0 + max_light);

    vec3 c = pow(col_adjusted, vec3(s)) * T;
    // vec3 c = col_adjusted * T;
    // vec3 c = sqrt(col_adjusted) * T;
    // vec3 c = /*col_adjusted * */col_adjusted * T;

    return c;
    // float sum_col = color.r + color.g + color.b;
    // return /*srgb_to_linear*/(/*0.5*//*0.125 * */vec3(pow(color.x, gamma), pow(color.y, gamma), pow(color.z, gamma)));
}

/*
float depth_at(vec2 uv) {
    float buf_depth = texture(src_depth, uv).x;
    vec4 clip_space = vec4(uv * 2.0 - 1.0, buf_depth, 1.0);
    vec4 view_space = proj_mat_inv * clip_space;
    view_space /= view_space.w;
    return -view_space.z;
}

vec3 wpos_at(vec2 uv) {
    float buf_depth = texture(src_depth, uv).x * 2.0 - 1.0;
    mat4 inv = view_mat_inv * proj_mat_inv;//inverse(all_mat);
    vec4 clip_space = vec4(uv * 2.0 - 1.0, buf_depth, 1.0);
    vec4 view_space = inv * clip_space;
    view_space /= view_space.w;
    if (buf_depth == 1.0) {
        vec3 direction = normalize(view_space.xyz);
        return direction.xyz * 100000.0 + cam_pos.xyz;
    } else {
        return view_space.xyz;
    }
}
*/

vec3 lms_color(vec3 rgb) {
    return vec3(
        (17.8824 * rgb.r) + (43.5161 * rgb.g) + (4.11935 * rgb.b),
        (3.45565 * rgb.r) + (27.1554 * rgb.g) + (3.86714 * rgb.b),
        (0.0299566 * rgb.r) + (0.184309 * rgb.g) + (1.46709 * rgb.b)
    );
}

vec3 correct(vec3 rgb, vec3 dlms) {
    vec3 err = rgb - vec3(
        (0.0809444479 * dlms.r) + (-0.130504409 * dlms.g) + (0.116721066 * dlms.b),
        (-0.0102485335 * dlms.r) + (0.0540193266 * dlms.g) + (-0.113614708 * dlms.b),
        (-0.000365296938 * dlms.r) + (-0.00412161469 * dlms.g) + (0.693511405 * dlms.b)
    );
    vec3 correction = vec3(
        0.0,
        err.r * 0.7 + err.g * 1.0,
        err.r * 0.7 + err.b * 1.0
    );
    return rgb + correction;
}

#define COLOR_NONE 0
#define COLOR_PROTANOPIA 1
#define COLOR_DEUTERANOPIA 2
#define COLOR_TRITANOPIA 3

#define COLOR_CORRECTION COLOR_NONE

vec3 color_correction(vec3 rgb) {
    vec3 lms = lms_color(rgb);
    #if (COLOR_CORRECTION == COLOR_PROTANOPIA)
        vec3 rgb_new = correct(rgb, vec3(
            0.0 * lms.r + 2.02344 * lms.g + -2.52581 * lms.b,
            0.0 * lms.r + 1.0 * lms.g + 0.0 * lms.b,
            0.0 * lms.r + 0.0 * lms.g + 1.0 * lms.b
        ));
    #elif (COLOR_CORRECTION == COLOR_DEUTERANOPIA)
        vec3 rgb_new = correct(rgb, vec3(
            1.0 * lms.r + 0.0 * lms.g + 0.0 * lms.b,
            0.494207 * lms.r + 0.0 * lms.g + 1.24827 * lms.b,
            0.0 * lms.r + 0.0 * lms.g + 1.0 * lms.b
        ));
    #elif (COLOR_CORRECTION == COLOR_TRITANOPIA)
        vec3 rgb_new = correct(rgb, vec3(
            1.0 * lms.r + 0.0 * lms.g + 0.0 * lms.b,
            0.0 * lms.r + 1.0 * lms.g + 0.0 * lms.b,
            -0.395913 * lms.r + 0.801109 * lms.g + 0.0 * lms.b
        ));
    #else
        vec3 rgb_new = rgb;
    #endif

    return rgb_new;
}

void main() {
    vec2 uv = (f_pos + 1.0) * 0.5;

    /* if (medium.x == 1u) {
        uv = clamp(uv + vec2(sin(uv.y * 16.0 + tick.x), sin(uv.x * 24.0 + tick.x)) * 0.005, 0, 1);
    } */

    vec2 c_uv = vec2(0.5);//uv;//vec2(0.5);//uv;
    vec2 delta = /*sqrt*//*sqrt(2.0) / 2.0*//*sqrt(2.0) / 2.0*//*0.5 - */min(uv, 1.0 - uv);//min(uv * (1.0 - uv), 0.25) * 2.0;
    // delta = /*sqrt(2.0) / 2.0 - */sqrt(vec2(dot(delta, delta)));
    // delta = 0.5 - vec2(min(delta.x, delta.y));
    delta = vec2(0.25);//vec2(dot(/*0.5 - */delta, /*0.5 - */delta));//vec2(min(delta.x, delta.y));//sqrt(2.0) * (0.5 - vec2(min(delta.x, delta.y)));
    // delta = vec2(sqrt(dot(delta, delta)));
    // vec2 delta = /*sqrt*//*sqrt(2.0) / 2.0*//*sqrt(2.0) / 2.0*/1.0 - vec2(sqrt(dot(uv, 1.0 - uv)));//min(uv * (1.0 - uv), 0.25) * 2.0;
    // float delta = /*sqrt*//*sqrt(2.0) / 2.0*//*sqrt(2.0) / 2.0*/1.0 - (dot(uv - 0.5, uv - 0.5));//0.01;//25;
    // vec2 delta = /*sqrt*//*sqrt(2.0) / 2.0*//*sqrt(2.0) / 2.0*/sqrt(uv * (1.0 - uv));//min(uv * (1.0 - uv), 0.25) * 2.0;

    // float bright_color0 = rel_luminance(texelFetch/*texture*/(src_color, ivec2(clamp(c_uv + vec2(0.0, 0.0), 0.0, 1.0) * screen_res.xy/* / 50*/)/* * 50*/, 0).rgb);
    // float bright_color1 = rel_luminance(texelFetch/*texture*/(src_color, ivec2(clamp(c_uv + vec2(delta.x, delta.y), 0.0, 1.0) * screen_res.xy/* / 50*/)/* * 50*/, 0).rgb);
    // float bright_color2 = rel_luminance(texelFetch/*texture*/(src_color, ivec2(clamp(c_uv + vec2(delta.x, -delta.y), 0.0, 1.0) * screen_res.xy/* / 50*/)/* * 50*/, 0).rgb);
    // float bright_color3 = rel_luminance(texelFetch/*texture*/(src_color, ivec2(clamp(c_uv + vec2(-delta.x, delta.y), 0.0, 1.0) * screen_res.xy/* / 50*/)/* * 50*/, 0).rgb);
    // float bright_color4 = rel_luminance(texelFetch/*texture*/(src_color, ivec2(clamp(c_uv + vec2(-delta.x, -delta.y), 0.0, 1.0) * screen_res.xy/* / 50*/)/* * 50*/, 0).rgb);

    // float bright_color0 = rel_luminance(texture(src_color, /*ivec2*/(clamp(c_uv + vec2(0.0, 0.0), 0.0, 1.0)/* * screen_res.xy*//* / 50*/)/* * 50*/, 0).rgb);
    // float bright_color1 = rel_luminance(texture(src_color, /*ivec2*/(clamp(c_uv + vec2(delta, delta), 0.0, 1.0)/* * screen_res.xy*//* / 50*/)/* * 50*/, 0).rgb);
    // float bright_color2 = rel_luminance(texture(src_color, /*ivec2*/(clamp(c_uv + vec2(delta, -delta), 0.0, 1.0)/* * screen_res.xy*//* / 50*/)/* * 50*/, 0).rgb);
    // float bright_color3 = rel_luminance(texture(src_color, /*ivec2*/(clamp(c_uv + vec2(-delta, delta), 0.0, 1.0)/* * screen_res.xy*//* / 50*/)/* * 50*/, 0).rgb);
    // float bright_color4 = rel_luminance(texture(src_color, /*ivec2*/(clamp(c_uv + vec2(-delta, -delta), 0.0, 1.0)/* * screen_res.xy*//* / 50*/)/* * 50*/, 0).rgb);

    // float bright_color = max(bright_color0, max(bright_color1, max(bright_color2, max(bright_color3, bright_color4))));// / 2.0;// / 5.0;

    // float bright_color = (bright_color0 + bright_color1 + bright_color2 + bright_color3 + bright_color4) / 5.0;

    vec4 aa_color = aa_apply(src_color, uv * screen_res.xy, screen_res.xy);

    // Tonemapping
    float exposure_offset = 1.0;
    // Adding an in-code offset to gamma and exposure let us have more precise control over the game's look
    float gamma_offset = 0.3;
    aa_color.rgb = vec3(1.0) - exp(-aa_color.rgb * (gamma_exposure.y + exposure_offset));
    // gamma correction
    aa_color.rgb = pow(aa_color.rgb, vec3(gamma_exposure.x + gamma_offset));

    // Apply colour correction (color blindness filter)
    aa_color.rgb = color_correction(clamp(aa_color.rgb, vec3(0), vec3(1)));


    /*
    // Apply clouds to `aa_color`
    #if (CLOUD_MODE != CLOUD_MODE_NONE)
        vec3 wpos = wpos_at(uv);
        float dist = distance(wpos, cam_pos.xyz);
        vec3 dir = (wpos - cam_pos.xyz) / dist;

        aa_color.rgb = get_cloud_color(aa_color.rgb, dir, cam_pos.xyz, time_of_day.x, dist, 1.0);
    #endif
    */

    // aa_color.rgb = (wpos + focus_off.xyz) / vec3(32768, 32768, /*view_distance.w*/2048);
    // aa_color.rgb = mod((wpos + focus_off.xyz), vec3(32768, 32768, view_distance.w)) / vec3(32768, 32768, view_distance.w);// / vec3(32768, 32768, view_distance.w);
    // aa_color.rgb = mod((wpos + focus_off.xyz), vec3(32, 32, 16)) / vec3(32, 32, 16);// / vec3(32768, 32768, view_distance.w);
    // aa_color.rgb = focus_off.xyz / vec3(32768, 32768, view_distance.w);

    /* aa_color.rgb = wpos / 10000.0; */

    /* aa_color.rgb = vec3((texture(src_depth, uv).x - 0.99) * 100.0); */

    /* aa_color.rgb = vec3((dist - 100000) / 300000.0, 1, 1); */

    /* vec3 scatter_color = get_sun_color() * get_sun_brightness() + get_moon_color() * get_moon_brightness(); */

    /* aa_color.rgb += cloud_color.rgb * scatter_color;//mix(aa_color, vec4(cloud_color.rgb * scatter_color, 1), cloud_color.a); */

    // aa_color.rgb = illuminate(1.0 - 1.0 / (1.0 + bright_color), normalize(cam_pos.xyz - focus_pos.xyz), /*vec3 max_light, */vec3(0.0), aa_color.rgb);

    //vec4 hsva_color = vec4(rgb2hsv(fxaa_color.rgb), fxaa_color.a);
    //hsva_color.y *= 1.45;
    //hsva_color.z *= 0.85;
    //hsva_color.z = 1.0 - 1.0 / (1.0 * hsva_color.z + 1.0);
    //vec4 final_color = vec4(hsv2rgb(hsva_color.rgb), hsva_color.a);

    tgt_color = vec4(aa_color.rgb, 1);
}
