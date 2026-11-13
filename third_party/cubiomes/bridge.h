#ifndef CUBIOMES_BRIDGE_H_
#define CUBIOMES_BRIDGE_H_

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct McWorldHandle McWorldHandle;

typedef struct McStructureHit {
    int32_t stype;
    int32_t x;
    int32_t z;
    int32_t variant; /* bit 0: has_end_ship / abandoned / igloo_basement / giant_portal */
    int32_t biome;
} McStructureHit;

/* Structure bitmasks matching enum StructureType indices requested by UI */
enum McStructMask {
    MC_MASK_VILLAGE        = 1u << 0,
    MC_MASK_DESERT_PYRAMID = 1u << 1,
    MC_MASK_JUNGLE_TEMPLE  = 1u << 2,
    MC_MASK_SWAMP_HUT      = 1u << 3,
    MC_MASK_IGLOO          = 1u << 4,
    MC_MASK_MONUMENT       = 1u << 5,
    MC_MASK_MANSION        = 1u << 6,
    MC_MASK_OUTPOST        = 1u << 7,
    MC_MASK_RUINED_PORTAL  = 1u << 8,
    MC_MASK_ANCIENT_CITY   = 1u << 9,
    MC_MASK_TRAIL_RUINS    = 1u << 10,
    MC_MASK_TRIAL_CHAMBERS = 1u << 11,
    MC_MASK_FORTRESS       = 1u << 12,
    MC_MASK_BASTION        = 1u << 13,
    MC_MASK_END_CITY       = 1u << 14,
    MC_MASK_END_GATEWAY    = 1u << 15,
};

enum McMapStyle {
    MC_STYLE_FLAT      = 0,
    MC_STYLE_TOPO_3D   = 1,
    MC_STYLE_MC_MAP    = 2,
};

McWorldHandle *mc_world_new(uint64_t seed, int32_t large_biomes);
void mc_world_free(McWorldHandle *h);

/*
 * Rasterize a rectangular tile at block_x0, block_z0 with resolution 'scale' blocks/pixel
 * (supported scale: 4, 16, 64) into out_rgba (width * height * 4 bytes).
 */
void mc_render_tile(
    const McWorldHandle *h,
    int32_t dim,
    int32_t scale,
    int32_t block_x0,
    int32_t block_z0,
    int32_t w,
    int32_t h_px,
    int32_t map_style,
    uint8_t *out_rgba
);

/* Sample exact biome and height proxy at 1:1 block coordinates. Returns biome_id. */
int32_t mc_sample_point(
    const McWorldHandle *h,
    int32_t dim,
    int32_t block_x,
    int32_t block_z,
    float *out_height,
    char *out_name,
    int32_t name_cap
);

void mc_get_spawn(McWorldHandle *h, int32_t *out_x, int32_t *out_z);
int32_t mc_get_strongholds(McWorldHandle *h, McStructureHit *out, int32_t max_out);
int32_t mc_get_end_gateways(McWorldHandle *h, McStructureHit *out_inner_20, McStructureHit *out_linked_20);

/* Scan structures intersecting world block box [min_bx..max_bx, min_bz..max_bz]. */
int32_t mc_scan_structures(
    McWorldHandle *h,
    int32_t dim,
    int32_t min_bx,
    int32_t min_bz,
    int32_t max_bx,
    int32_t max_bz,
    uint32_t mask,
    McStructureHit *out,
    int32_t max_out
);

#ifdef __cplusplus
}
#endif

#endif /* CUBIOMES_BRIDGE_H_ */
