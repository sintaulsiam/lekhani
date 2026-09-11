#ifndef LEKHANI_FFI_H
#define LEKHANI_FFI_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct LekhaniEngineContext LekhaniEngineContext;

LekhaniEngineContext *lekhani_engine_new(void);
void lekhani_engine_free(LekhaniEngineContext *ctx);

bool lekhani_engine_set_layout(LekhaniEngineContext *ctx, const char *layout_name);
void lekhani_engine_reload_config(LekhaniEngineContext *ctx);
void lekhani_engine_reset(LekhaniEngineContext *ctx);

bool lekhani_engine_process_key(
    LekhaniEngineContext *ctx,
    uint32_t keyval,
    uint32_t keycode,
    uint32_t state_mask,
    bool is_release
);

const char *lekhani_engine_get_commit_text(LekhaniEngineContext *ctx);
const char *lekhani_engine_get_preedit_text(LekhaniEngineContext *ctx);
const char *lekhani_engine_get_auxiliary_text(LekhaniEngineContext *ctx);

size_t lekhani_engine_get_candidate_count(LekhaniEngineContext *ctx);
const char *lekhani_engine_get_candidate_at(LekhaniEngineContext *ctx, size_t index);
size_t lekhani_engine_get_selected_candidate_index(LekhaniEngineContext *ctx);

bool lekhani_engine_select_candidate(LekhaniEngineContext *ctx, size_t index);
bool lekhani_engine_is_active(LekhaniEngineContext *ctx);

#ifdef __cplusplus
}
#endif

#endif // LEKHANI_FFI_H
