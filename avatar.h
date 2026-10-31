#ifndef ABSTRACTAVATAR_H
#define ABSTRACTAVATAR_H
#include <stdint.h>
#include <stddef.h>
#ifdef __cplusplus
extern "C" {
#endif
/* ABI1. Keys are durable; style indices are resolved from this catalog at runtime.
 * Fields0..4: key,name,description,watch reference,family. String pointers static.
 * Pose0..6: Ready,Thinking,Working,Listening,Speaking,Waiting,Error.
 * Palette material0..2: Warm,Eye,Dim; index0..255; rgba0xRRGGBBAA.
 * Set all768 shades before draw. Dimensions1..512; at most16 live handles.
 * Negative results are explicit errors. create returns0 on refusal.
 * Pixel pointers are borrowed until destroy; copy before another render/destroy.
 * Native hosts serialize palette/draw and concurrent borrowed-pointer access.
 */
uint32_t avatar_abi_version(void);
uint32_t avatar_style_count(void);
const uint8_t *avatar_style_field_ptr(uint32_t style,uint32_t field);
size_t avatar_style_field_len(uint32_t style,uint32_t field);
uint32_t avatar_create(uint32_t width,uint32_t height);
int32_t avatar_set_shade(uint32_t handle,uint32_t material,uint32_t index,uint32_t rgba);
int32_t avatar_set_ground(uint32_t handle,uint32_t rgba);
/* Optional normalized Ready gaze; other poses ignore it. Host expires activity. */
int32_t avatar_set_gaze(uint32_t handle,float x,float y);
/* Eight measured normalized bands; clear restores scalar-level defaults. */
int32_t avatar_set_bands(uint32_t handle,float b0,float b1,float b2,float b3,float b4,float b5,float b6,float b7);
int32_t avatar_clear_bands(uint32_t handle);
int32_t avatar_render(uint32_t handle,uint32_t style,uint32_t pose,float level,float seconds,uint32_t transparent);
/* Atomic render/copy under the registry lock. Caller owns a writable output
 * allocation of capacity bytes, disjoint from renderer buffers and other calls.
 * Needs width*height*4 bytes. -6 means null or insufficient output capacity.
 */
int32_t avatar_render_copy(uint32_t handle,uint32_t style,uint32_t pose,float level,float seconds,uint32_t transparent,uint8_t *output,size_t capacity);
const uint8_t *avatar_pixels(uint32_t handle);
size_t avatar_pixels_len(uint32_t handle);
int32_t avatar_destroy(uint32_t handle);
#ifdef __cplusplus
}
#endif
#endif
