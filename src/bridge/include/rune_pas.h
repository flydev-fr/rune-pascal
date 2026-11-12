// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
#ifndef RUNE_PAS_H
#define RUNE_PAS_H
#include <stdint.h>
#ifdef _WIN32
#define RUNE_PAS_API __declspec(dllimport)
#define RUNE_PAS_CALL __cdecl
#else
#define RUNE_PAS_API
#define RUNE_PAS_CALL
#endif
#ifdef __cplusplus
extern "C" {
#endif

#define RUNE_PAS_ABI_VERSION_CURRENT 1
#define RUNE_PAS_OK 0
#define RUNE_PAS_INVALID_ARGUMENT (-1)
#define RUNE_PAS_ALREADY_RUN (-2)
#define RUNE_PAS_RUNTIME_ERROR (-3)
#define RUNE_PAS_PANIC (-4)
#define RUNE_APPEARANCE_AUTO 0
#define RUNE_APPEARANCE_LIGHT 1
#define RUNE_APPEARANCE_DARK 2
#define RUNE_EVENT_READY 1
#define RUNE_EVENT_CHANGED 2
#define RUNE_EVENT_CLOSED 3

/* Natural C alignment, 32 bytes. Do not pack. All booleans are uint32_t (0/1).
   accent: 0..5; speed: finite 0..1 (0.5 = 1x); appearance: constants above.
   Toggle values are demo state only: they do not change OS settings. */
typedef struct RuneSettings {
    uint32_t struct_size;
    uint32_t appearance;
    uint32_t accent;
    uint32_t launch_at_login;
    uint32_t menu_bar;
    uint32_t sounds;
    float speed;
    uint32_t reduce_motion;
} RuneSettings;

/* Runs on the run() calling thread. Borrowed snapshot: copy it before returning.
   READY follows first presentation; CHANGED reports the latest presented state
   (not every intermediate input); CLOSED follows normal event-loop shutdown.
   Return nonzero from READY/CHANGED to request quit. CLOSED return is ignored.
   Callback/user_data must remain valid until run returns. Do not throw exceptions,
   longjmp, unload the DLL, or reenter run from a callback. */
typedef int32_t (RUNE_PAS_CALL *RuneSettingsCallback)(
    uint32_t event_kind, const RuneSettings *settings, void *user_data);

RUNE_PAS_API uint32_t RUNE_PAS_CALL rune_pas_abi_version(void);
/* out must address a complete writable, aligned RuneSettings. */
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_settings_default(RuneSettings *out);
/* UTF-8 error for the calling thread. Required capacity includes NUL.
   Null buffer/zero capacity is a sizing query; short buffers are NUL-terminated.
   Error text is set on failure and cleared at run entry. */
RUNE_PAS_API uint32_t RUNE_PAS_CALL rune_pas_last_error(char *buffer, uint32_t capacity);
/* Blocking native window; call on the process main thread, once per process.
   config NULL = defaults; otherwise initialized storage with matching struct_size.
   callback NULL = no notifications. config is copied and may be released on callback.
   Does not consume host command-line flags. Keep the DLL loaded until run returns.
   Cannot be embedded in an existing VCL/LCL loop with this first API.
   Invalid arguments do not consume the one allowed run. */
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_settings_run(
    const RuneSettings *config, RuneSettingsCallback callback, void *user_data);


/* Lab ABI extension; additive to v1. All scene IDs below are stable. */
#define RUNE_PAS_NOT_RUNNING (-5)
#define RUNE_LAB_MAX_KNOBS 16
#define RUNE_LAB_SCENE_LAMP 0
#define RUNE_LAB_SCENE_FLASHLIGHT 1
#define RUNE_LAB_SCENE_GI 2
#define RUNE_LAB_SCENE_KEYS 3
#define RUNE_LAB_SCENE_GODRAYS 4
#define RUNE_LAB_SCENE_GLOW 5
#define RUNE_LAB_SCENE_PULSES 6
#define RUNE_LAB_SCENE_ELECTRIC 7
#define RUNE_LAB_SCENE_FIRE 8
#define RUNE_LAB_SCENE_MOTION 9
#define RUNE_LAB_SCENE_LENS 10
#define RUNE_LAB_SCENE_EXTRAS 11
#define RUNE_LAB_KNOB_SLIDER 0
#define RUNE_LAB_KNOB_TOGGLE 1
#define RUNE_LAB_KNOB_SWATCH 2

/* Natural alignment, 80 bytes. knob_count=0 selects scene defaults.
   Otherwise knob_count must match rune_lab_knob_count(scene), and every
   value must be finite and inside the reported range. Toggles and swatches
   require integer values. Unused array entries are ignored.
   Callback snapshots always contain all of the selected scene's knob values. */
typedef struct RuneLabConfig {
    uint32_t struct_size;
    uint32_t scene;
    uint32_t reduce_motion;
    uint32_t knob_count;
    float knob_values[RUNE_LAB_MAX_KNOBS];
} RuneLabConfig;

/* Natural alignment, 116 bytes. UTF-8 names, NUL-terminated. */
typedef struct RuneLabKnobInfo {
    uint32_t struct_size;
    float min_value;
    float max_value;
    float default_value;
    uint32_t kind;
    char name[32];
    char label[64];
} RuneLabKnobInfo;

/* Same lifetime/calling convention as RuneSettingsCallback. READY/CHANGED/CLOSED
   constants are shared. CHANGED reports presented scene, reduced-motion and knob
   changes, not pointer movement, transient effects or frame timing.
   Nonzero callback result requests quit. May call set_config/quit from callbacks;
   do not reenter run. Keep the DLL loaded for the process lifetime. */
typedef int32_t (RUNE_PAS_CALL *RuneLabCallback)(
    uint32_t event_kind, const RuneLabConfig *config, void *user_data);

RUNE_PAS_API uint32_t RUNE_PAS_CALL rune_lab_scene_count(void);
/* Required UTF-8 capacity including NUL; 0 on invalid scene. NULL buffer allowed.
   A non-null buffer must be writable for capacity bytes. */
RUNE_PAS_API uint32_t RUNE_PAS_CALL rune_lab_scene_name(uint32_t scene, char *buffer, uint32_t capacity);
RUNE_PAS_API uint32_t RUNE_PAS_CALL rune_lab_knob_count(uint32_t scene);
/* out must address the complete writable output struct (size set by the DLL). */
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_lab_knob_info(uint32_t scene, uint32_t index, RuneLabKnobInfo *out);
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_lab_default(RuneLabConfig *out);
/* Blocking, main thread. config NULL=defaults, callback NULL=no notifications.
   Shares the one-event-loop-per-process restriction with Settings.
   Host must mask floating-point exceptions while run executes. */
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_lab_run(
    const RuneLabConfig *config, RuneLabCallback callback, void *user_data);
/* Thread-safe queued update, config copied before return; non-null required.
   0 means queued, not necessarily presented. Calls racing with quit may be dropped.
   No lock is held during callbacks. knob_count=0 resets the chosen scene's knobs. */
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_lab_set_config(const RuneLabConfig *config);
/* Thread-safe queued quit. set_config/quit return NOT_RUNNING outside run. */
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_lab_quit(void);


/* Generic UI tree API. Additive to ABI 1; all records use natural alignment. */
#define RUNE_UI_COLUMN 1
#define RUNE_UI_ROW 2
#define RUNE_UI_TEXT 3
#define RUNE_UI_BUTTON 4
#define RUNE_UI_INPUT 5
#define RUNE_UI_CHECKBOX 6
#define RUNE_UI_DOCUMENT 7
#define RUNE_UI_NAV_ITEM 8
/* Combobox text: 1..64 nonempty newline-separated labels. */
#define RUNE_UI_COMBOBOX 9
#define RUNE_UI_CENTER_X 64
#define RUNE_UI_MAX_WIDTH 128
#define RUNE_UI_BORDER 256
/* Column directly under root, one per tree. Centered over a scrim, focus trapped.
   Escape/outside click emits DISMISS; host removes the modal on its next commit. */
#define RUNE_UI_MODAL 512
/* Return activates this button, scoped to the active modal. */
#define RUNE_UI_DEFAULT_BUTTON 1024
#define RUNE_UI_DISMISS 16
/* Embedded host requests a plain UTF-8 clipboard write after Ctrl+C. id/value=0.
   The callback text is borrowed only during the call; VCL writes CF_UNICODETEXT. */
#define RUNE_UI_COPY 17
/* id = combobox node, value = zero-based option index. Host accepts via set_choice. */
#define RUNE_UI_CHOICE 18
#define RUNE_UI_LINK 15
#define RUNE_UI_GROW 1
#define RUNE_UI_FULL_WIDTH 2
#define RUNE_UI_FULL_HEIGHT 4
#define RUNE_UI_SCROLL 8
#define RUNE_UI_WRAP 16
#define RUNE_UI_ANIMATE 32
#define RUNE_UI_CLICK 10
#define RUNE_UI_EDIT 11
#define RUNE_UI_SUBMIT 12
#define RUNE_UI_CHECK 13
#define RUNE_UI_COMMITTED 14

/* 44 bytes, align 4. width/height=-1 means auto; other sizes in points.
   Colors: sRGB 0xRRGGBBAA. flags are ORed from RUNE_UI_* above. */
typedef struct RuneUiStyle {
    uint32_t struct_size;
    uint32_t flags;
    float width;
    float height;
    float gap;
    float padding;
    float radius;
    float font_size;
    uint32_t font_weight;
    uint32_t background_rgba;
    uint32_t foreground_rgba;
} RuneUiStyle;

/* Main UI thread. READY/CLOSED shared with demos. COMMITTED gives the latest
   presented revision in value; coalesced commits may skip revisions.
   CLICK has node ID; EDIT/SUBMIT have borrowed UTF-8 text; CHECK value=0/1.
   Copy text before returning. Nonzero requests quit. Callbacks may rebuild/commit
   or quit, but must not reenter run, throw, retain pointers, or unload the DLL. */
typedef int32_t (RUNE_PAS_CALL *RuneUiCallback)(uint32_t kind, uint32_t node_id,
    uint32_t value, const char *text, void *user_data);

/* One global transaction, owned by the thread that begins it. Repeated begin on
   that thread replaces staging. Other threads cannot mutate/commit/abort it.
   Failed operations do not replace the committed tree. Abort discards staging.
   Pre-run and callback use supported. No general concurrent app/builder support. */
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_begin(void);
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_abort(void);
/* Caller-owned stable nonzero IDs, unique per tree. Parent must already exist;
   parent=0 creates the one row/column root. Only rows/columns have children.
   Text is copied UTF-8, NULL=empty. Limits: 2048 nodes, depth 64, 64 KiB per
   string, 2 MiB total tree text. Preserve IDs across commits to retain focus. */
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_add(uint32_t parent, uint32_t id, uint32_t kind, const char *text);
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_style_default(RuneUiStyle *out);
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_set_style(uint32_t id, const RuneUiStyle *style);
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_set_checked(uint32_t id, uint32_t checked);
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_set_choice(uint32_t id, uint32_t index);
/* Validates and atomically publishes the complete tree. During run, queues it;
   COMMITTED confirms presentation. Updates racing with quit may be dropped.
   Checkbox state is controlled by the host: rebuild after CHECK to accept it.
   Inputs retain selection/undo if their text is unchanged when committed. */
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_commit(void);
/* Commit first. Blocking main-thread native window, at most one run (UI/Settings/
   Lab) per process. Mask floating-point exceptions during run; DLL retained for
   process lifetime. Title copied UTF-8 <=1024 bytes, dimensions 240..8192 points. */
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_run(const char *title, uint32_t width,
    uint32_t height, RuneUiCallback callback, void *user_data);
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_quit(void);


/* Shader effects attached to a UI button inside the current transaction. */
#define RUNE_FX_NONE 0
#define RUNE_FX_GLOW 1
#define RUNE_FX_ELECTRIC 2
#define RUNE_FX_FIRE 3
#define RUNE_FX_SHIMMER 4
#define RUNE_FX_METAL 5
#define RUNE_FX_PLASTIC 6
#define RUNE_FX_PULSE 7
#define RUNE_FX_GLASS 8
#define RUNE_FX_HOVER_ONLY 1
#define RUNE_FX_REDUCED_MOTION 2
/* 32 bytes, align 4. Use effect_default for kind-specific defaults.
   Color: sRGB RRGGBBAA (alpha scales the effect); intensity 0..3 (fills/glass
   cap opacity at 1), speed 0..4 (0 freezes time), size 0..128 points.
   size = glow/pulse radius, electric spread, fire height, shimmer band,
   metal/plastic bevel or glass depth.
   parameter 0..1 = glow breathe, metal flow, plastic roughness, glass frost;
   unused by other effects. Physical fire/shimmer/glass keep their own colors.
   One effect per button. NONE removes it. Flags can be combined.
   Frozen/reduced/static effects do not keep an idle window ticking.
   Fire uses rune-fx's procedural flames; no fluid simulation in this bridge.
   Shared WGSL shaders from Lab, not arbitrary user-supplied shader code. */
typedef struct RuneUiEffect {
    uint32_t struct_size;
    uint32_t kind;
    uint32_t color_rgba;
    uint32_t flags;
    float intensity;
    float speed;
    float size;
    float parameter;
} RuneUiEffect;
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_effect_default(uint32_t kind, RuneUiEffect *out);
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_set_effect(uint32_t id, const RuneUiEffect *effect);


/* Experimental embedded Win32 host. VCL owns HWND and message pump.
   Main-thread-only; handles are thread-local numeric tokens, not pointers.
   Destroy the host BEFORE destroying/recreating its HWND. No winit event loop.
   Each host currently has its own GPU device. Tree builders are still serialized;
   commit publishes staging, then host_commit copies that tree to the chosen host.
   Prototype input: buttons/mouse, Tab, Enter, Space, arrows. No IME, clipboard paste,
   native accessibility adapter or complete edit-control integration yet. */
typedef struct RuneHostFrame {
    uint32_t struct_size;
    uint32_t flags; /* bit0: request another animation frame */
    double update_ms;
    double encode_ms; /* CPU encoding only: no GPU execution or present wait */
    double present_ms; /* acquire+submit+present CPU/wait time */
    double total_ms;
} RuneHostFrame;
typedef struct RuneBenchResult {
    uint32_t struct_size;
    uint32_t frames;
    double encode_median_ms;
    double encode_p95_ms;
    double encode_mean_ms;
    double elapsed_ms;
    double cpu_ms; /* reserved, currently zero; not a measured GPU/CPU time */
} RuneBenchResult;
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_host_create(uintptr_t hwnd, uint32_t width,
    uint32_t height, float scale, RuneUiCallback callback, void *user_data, uint32_t *out);
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_host_destroy(uint32_t host);
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_host_commit(uint32_t host);
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_host_resize(uint32_t host, uint32_t width, uint32_t height, float scale);
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_host_input(uint32_t host, uint32_t kind,
    float x, float y, uint32_t value, uint32_t flags);
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_host_frame(uint32_t host, RuneHostFrame *out);
/* Controlled native window benchmark: 60 warm-up frames then requested samples.
   CPU compositor encoding scope only. Continuously redraws even a static scene.
   No GPU timestamps: encode_ms is NOT GPU render duration. */
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_benchmark(uint32_t frames, RuneBenchResult *out);

/* Formatted document fragment produced by mormot.ext.markdown; native Rune rendering.
   Strings copied. Markup <=2 MiB, base is the local image directory, query highlights text.
   Only local images are loaded (up to 24); remote images retain their alt text.
   host_input kind 5 accepts VK_A/VK_C with flags bit1 Ctrl;
   copied text is delivered as RUNE_UI_COPY.
   host_input kind 7 scrolls at physical x/y; value is signed logical-pixel delta as uint32. */
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_set_document(uint32_t id,
    const char *markup, const char *base, const char *query);
/* Select installed font families (CSS family lists), on the host UI thread.
   No fonts are downloaded; missing families fall back to the next installed name. */
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_host_set_fonts(uint32_t host, const char *sans, const char *mono);

/* Optional transactional palette: 84 bytes, align 4; colors sRGB RRGGBBAA.
   Width 0..4; radii/padding 0..64 points, finite. Existing trees without a
   palette keep their previous generic control appearance. Strings copied. */
typedef struct RuneUiTheme {
    uint32_t struct_size;
    uint32_t background;
    uint32_t sidebar;
    uint32_t surface;
    uint32_t foreground;
    uint32_t muted;
    uint32_t border;
    uint32_t code_background;
    uint32_t code_border;
    uint32_t inline_code_background;
    uint32_t quote_background;
    uint32_t quote_border;
    uint32_t table_header;
    uint32_t link;
    uint32_t highlight;
    float border_width;
    float panel_radius;
    float control_radius;
    float code_radius;
    float code_padding;
    float inline_code_padding;
} RuneUiTheme;
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_theme_default(uint32_t dark, RuneUiTheme *out);
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_set_theme(const RuneUiTheme *theme);
RUNE_PAS_API int32_t RUNE_PAS_CALL rune_ui_set_hint(uint32_t id, const char *hint);

#ifdef __cplusplus
}
#endif
#endif
