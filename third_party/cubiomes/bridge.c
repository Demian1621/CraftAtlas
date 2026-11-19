#include "bridge.h"
#include "biomes.h"
#include "generator.h"
#include "finders.h"
#include "util.h"

#include <stdlib.h>
#include <string.h>
#include <math.h>

struct McWorldHandle {
    Generator g_ow;
    Generator g_net;
    Generator g_end;
    SurfaceNoise sn_end;
    uint64_t seed;
    int32_t mc_ver;
    int32_t large_biomes;
    uint8_t colors[256][3];
};

static int32_t resolve_cubiomes_mc_ver(int32_t code) {
    switch (code) {
        case 1182: return MC_1_18_2;
        case 1192: return MC_1_19_2;
        case 1194: return MC_1_19_4;
        case 1206: return MC_1_20_6;
        case 1211: return MC_1_21_1;
        case 1213: return MC_1_21_3;
        default:   return MC_NEWEST;
    }
}

static inline int32_t floor_div_i32(int32_t a, int32_t b) {
    int32_t q = a / b;
    int32_t r = a % b;
    if (r != 0 && ((r < 0) != (b < 0))) q--;
    return q;
}

static inline uint8_t clamp_u8(float v) {
    if (v <= 0.0f) return 0;
    if (v >= 255.0f) return 255;
    return (uint8_t)(v + 0.5f);
}

static void init_rich_palette(uint8_t colors[256][3]) {
    initBiomeColors(colors);

    /* Modern & vibrant tweaks matching Chunkbase / Cubiomes best legibility */
    colors[ocean][0] = 28; colors[ocean][1] = 68; colors[ocean][2] = 166;
    colors[deep_ocean][0] = 14; colors[deep_ocean][1] = 40; colors[deep_ocean][2] = 115;
    colors[warm_ocean][0] = 44; colors[warm_ocean][1] = 148; colors[warm_ocean][2] = 205;
    colors[lukewarm_ocean][0] = 36; colors[lukewarm_ocean][1] = 108; colors[lukewarm_ocean][2] = 186;
    colors[deep_lukewarm_ocean][0] = 22; colors[deep_lukewarm_ocean][1] = 72; colors[deep_lukewarm_ocean][2] = 152;
    colors[cold_ocean][0] = 32; colors[cold_ocean][1] = 74; colors[cold_ocean][2] = 156;
    colors[deep_cold_ocean][0] = 20; colors[deep_cold_ocean][1] = 48; colors[deep_cold_ocean][2] = 118;
    colors[frozen_ocean][0] = 112; colors[frozen_ocean][1] = 148; colors[frozen_ocean][2] = 206;
    colors[deep_frozen_ocean][0] = 68; colors[deep_frozen_ocean][1] = 102; colors[deep_frozen_ocean][2] = 172;
    colors[river][0] = 48; colors[river][1] = 122; colors[river][2] = 212;
    colors[frozen_river][0] = 145; colors[frozen_river][1] = 185; colors[frozen_river][2] = 232;

    colors[plains][0] = 136; colors[plains][1] = 182; colors[plains][2] = 92;
    colors[sunflower_plains][0] = 172; colors[sunflower_plains][1] = 202; colors[sunflower_plains][2] = 84;
    colors[forest][0] = 66; colors[forest][1] = 138; colors[forest][2] = 62;
    colors[flower_forest][0] = 88; colors[flower_forest][1] = 156; colors[flower_forest][2] = 84;
    colors[birch_forest][0] = 108; colors[birch_forest][1] = 164; colors[birch_forest][2] = 92;
    colors[old_growth_birch_forest][0] = 98; colors[old_growth_birch_forest][1] = 154; colors[old_growth_birch_forest][2] = 84;
    colors[dark_forest][0] = 52; colors[dark_forest][1] = 94; colors[dark_forest][2] = 44;
    colors[pale_garden][0] = 112; colors[pale_garden][1] = 118; colors[pale_garden][2] = 122;
    colors[cherry_grove][0] = 242; colors[cherry_grove][1] = 154; colors[cherry_grove][2] = 194;
    colors[meadow][0] = 114; colors[meadow][1] = 168; colors[meadow][2] = 110;
    colors[mangrove_swamp][0] = 42; colors[mangrove_swamp][1] = 92; colors[mangrove_swamp][2] = 74;
    colors[swamp][0] = 74; colors[swamp][1] = 106; colors[swamp][2] = 68;
    colors[jungle][0] = 68; colors[jungle][1] = 156; colors[jungle][2] = 44;
    colors[bamboo_jungle][0] = 110; colors[bamboo_jungle][1] = 168; colors[bamboo_jungle][2] = 42;
    colors[desert][0] = 236; colors[desert][1] = 210; colors[desert][2] = 148;
    colors[savanna][0] = 178; colors[savanna][1] = 172; colors[savanna][2] = 92;
    colors[badlands][0] = 204; colors[badlands][1] = 102; colors[badlands][2] = 54;
    colors[eroded_badlands][0] = 228; colors[eroded_badlands][1] = 114; colors[eroded_badlands][2] = 66;
    colors[wooded_badlands][0] = 168; colors[wooded_badlands][1] = 114; colors[wooded_badlands][2] = 68;
    colors[mushroom_fields][0] = 188; colors[mushroom_fields][1] = 56; colors[mushroom_fields][2] = 202;
    colors[snowy_plains][0] = 224; colors[snowy_plains][1] = 234; colors[snowy_plains][2] = 242;
    colors[ice_spikes][0] = 166; colors[ice_spikes][1] = 216; colors[ice_spikes][2] = 240;
    colors[jagged_peaks][0] = 172; colors[jagged_peaks][1] = 182; colors[jagged_peaks][2] = 196;
    colors[frozen_peaks][0] = 208; colors[frozen_peaks][1] = 226; colors[frozen_peaks][2] = 244;
    colors[stony_peaks][0] = 136; colors[stony_peaks][1] = 138; colors[stony_peaks][2] = 134;

    /* Nether */
    colors[nether_wastes][0] = 132; colors[nether_wastes][1] = 46; colors[nether_wastes][2] = 42;
    colors[crimson_forest][0] = 178; colors[crimson_forest][1] = 26; colors[crimson_forest][2] = 46;
    colors[warped_forest][0] = 22; colors[warped_forest][1] = 152; colors[warped_forest][2] = 134;
    colors[soul_sand_valley][0] = 78; colors[soul_sand_valley][1] = 128; colors[soul_sand_valley][2] = 136;
    colors[basalt_deltas][0] = 68; colors[basalt_deltas][1] = 64; colors[basalt_deltas][2] = 76;

    /* The End */
    colors[the_end][0] = 226; colors[the_end][1] = 220; colors[the_end][2] = 158;
    colors[end_highlands][0] = 228; colors[end_highlands][1] = 216; colors[end_highlands][2] = 148;
    colors[end_midlands][0] = 186; colors[end_midlands][1] = 172; colors[end_midlands][2] = 114;
    colors[end_barrens][0] = 114; colors[end_barrens][1] = 102; colors[end_barrens][2] = 104;
    colors[small_end_islands][0] = 22; colors[small_end_islands][1] = 16; colors[small_end_islands][2] = 36;
}

McWorldHandle *mc_world_new(uint64_t seed, int32_t mc_version_code, int32_t large_biomes) {
    McWorldHandle *h = (McWorldHandle *)calloc(1, sizeof(McWorldHandle));
    if (!h) return NULL;
    h->seed = seed;
    h->mc_ver = resolve_cubiomes_mc_ver(mc_version_code);
    h->large_biomes = large_biomes ? 1 : 0;

    uint32_t flags = FORCE_OCEAN_VARIANTS;
    if (large_biomes) flags |= LARGE_BIOMES;

    setupGenerator(&h->g_ow, h->mc_ver, flags);
    applySeed(&h->g_ow, DIM_OVERWORLD, seed);

    setupGenerator(&h->g_net, h->mc_ver, 0);
    applySeed(&h->g_net, DIM_NETHER, seed);

    setupGenerator(&h->g_end, h->mc_ver, 0);
    applySeed(&h->g_end, DIM_END, seed);
    initSurfaceNoise(&h->sn_end, DIM_END, seed);

    init_rich_palette(h->colors);
    return h;
}

void mc_world_free(McWorldHandle *h) {
    if (h) free(h);
}

static void render_overworld_tile(
    const McWorldHandle *h,
    int32_t scale,
    int32_t block_x0,
    int32_t block_z0,
    int32_t w,
    int32_t h_px,
    int32_t map_style,
    uint8_t *out_rgba
) {
    int32_t bw = w + 2;
    int32_t bh = h_px + 2;
    size_t count = (size_t)bw * (size_t)bh;

    int32_t *ids = (int32_t *)malloc(count * sizeof(int32_t));
    float *depth = (float *)malloc(count * sizeof(float));
    if (!ids || !depth) {
        free(ids);
        free(depth);
        return;
    }

    int32_t bx_start = block_x0 - scale;
    int32_t bz_start = block_z0 - scale;

    if (scale >= 16 && map_style == MC_STYLE_FLAT) {
        /* Ultra-fast pure biome batch pass */
        Range r;
        r.scale = scale;
        r.x = floor_div_i32(bx_start, scale);
        r.z = floor_div_i32(bz_start, scale);
        r.sx = bw;
        r.sz = bh;
        r.y = 16;
        r.sy = 1;
        genBiomes(&h->g_ow, ids, r);
        memset(depth, 0, count * sizeof(float));
    } else if (scale == 1) {
        /* Exact 1:1 Voronoi + bilinear 1:4 relief (~13x fewer biome noise calls per tile) */
        Range r1 = {1, bx_start, bz_start, bw, bh, 64, 1};
        Range s = getVoronoiSrcRange(r1);
        /* Expand 1 cell so bilinear depth interpolation always stays in bounds */
        int32_t cx0 = s.x - 1;
        int32_t cz0 = s.z - 1;
        int32_t cw = s.sx + 2;
        int32_t ch = s.sz + 2;
        size_t ccount = (size_t)cw * (size_t)ch;

        int32_t *c_ids = (int32_t *)malloc(ccount * sizeof(int32_t));
        float *c_depth = (float *)malloc(ccount * sizeof(float));
        if (!c_ids || !c_depth) {
            free(c_ids);
            free(c_depth);
            free(ids);
            free(depth);
            return;
        }

        for (int32_t cj = 0; cj < ch; cj++) {
            int32_t bz4 = cz0 + cj;
            for (int32_t ci = 0; ci < cw; ci++) {
                int32_t bx4 = cx0 + ci;
                int64_t np[6];
                int id = sampleBiomeNoise(&h->g_ow.bn, np, bx4, 16, bz4, NULL, 0);
                size_t cidx = (size_t)cj * (size_t)cw + (size_t)ci;
                c_ids[cidx] = id;
                c_depth[cidx] = ((float)np[NP_DEPTH] / 10000.0f) * 2.2f;
            }
        }

        for (int32_t j = 0; j < bh; j++) {
            int32_t bz = bz_start + j;
            int32_t gz = floor_div_i32(bz - 2, 4);
            float fz = ((float)(bz - 2 - gz * 4)) * 0.25f;
            int32_t rz = gz - cz0;
            if (rz < 0) rz = 0;
            if (rz > ch - 2) rz = ch - 2;

            for (int32_t i = 0; i < bw; i++) {
                int32_t bx = bx_start + i;
                int vx, vy, vz;
                voronoiAccess3D(h->g_ow.sha, bx, 64, bz, &vx, &vy, &vz);
                int32_t lx = vx - cx0;
                int32_t lz = vz - cz0;
                if (lx < 0) lx = 0; else if (lx >= cw) lx = cw - 1;
                if (lz < 0) lz = 0; else if (lz >= ch) lz = ch - 1;

                size_t idx = (size_t)j * (size_t)bw + (size_t)i;
                ids[idx] = c_ids[(size_t)lz * (size_t)cw + (size_t)lx];

                int32_t gx = floor_div_i32(bx - 2, 4);
                float fx = ((float)(bx - 2 - gx * 4)) * 0.25f;
                int32_t rx = gx - cx0;
                if (rx < 0) rx = 0;
                if (rx > cw - 2) rx = cw - 2;

                float d00 = c_depth[(size_t)rz * (size_t)cw + (size_t)rx];
                float d10 = c_depth[(size_t)rz * (size_t)cw + (size_t)(rx + 1)];
                float d01 = c_depth[(size_t)(rz + 1) * (size_t)cw + (size_t)rx];
                float d11 = c_depth[(size_t)(rz + 1) * (size_t)cw + (size_t)(rx + 1)];
                float d0 = d00 + (d10 - d00) * fx;
                float d1 = d01 + (d11 - d01) * fx;
                depth[idx] = d0 + (d1 - d0) * fz;
            }
        }

        free(c_ids);
        free(c_depth);
    } else {
        /* 2D Biome + Depth elevation noise pass (scale 4, 16, 64, 256) */
        for (int32_t j = 0; j < bh; j++) {
            int32_t bz = floor_div_i32(bz_start + j * scale, 4);
            for (int32_t i = 0; i < bw; i++) {
                int32_t bx = floor_div_i32(bx_start + i * scale, 4);
                int64_t np[6];
                int id = sampleBiomeNoise(&h->g_ow.bn, np, bx, 16, bz, NULL, 0);
                size_t idx = (size_t)j * (size_t)bw + (size_t)i;
                ids[idx] = id;
                depth[idx] = ((float)np[NP_DEPTH] / 10000.0f) * 2.2f;
            }
        }
    }

    for (int32_t j = 0; j < h_px; j++) {
        for (int32_t i = 0; i < w; i++) {
            int32_t ix = i + 1;
            int32_t jz = j + 1;
            size_t c = (size_t)jz * (size_t)bw + (size_t)ix;
            int id = ids[c];
            if (id < 0 || id >= 256) id = plains;

            float r_col = (float)h->colors[id][0];
            float g_col = (float)h->colors[id][1];
            float b_col = (float)h->colors[id][2];

            float d_c = depth[c];
            int ocean_like = isOceanic(id) || id == river || id == frozen_river;

            if (map_style == MC_STYLE_TOPO_3D) {
                float dx = depth[c + 1] - depth[c - 1];
                float dz = depth[c + bw] - depth[c - bw];
                float shade = 1.0f + (-dx * 1.45f - dz * 1.45f);
                if (ocean_like) {
                    /* Coastal depth gradient instead of flat blue blob */
                    float bath = 0.88f + fmaxf(-0.22f, fminf(0.18f, d_c * 0.20f));
                    shade = bath + (-dx * 0.35f - dz * 0.35f);
                } else {
                    /* Soft elevation tint */
                    shade += fmaxf(-0.10f, fminf(0.16f, (d_c - 0.10f) * 0.14f));
                }
                if (shade < 0.62f) shade = 0.62f;
                if (shade > 1.32f) shade = 1.32f;

                /* Subtle biome boundary darken for crisp Chunkbase readability */
                if (ids[c - 1] != id || ids[c + 1] != id || ids[c - bw] != id || ids[c + bw] != id) {
                    shade *= 0.90f;
                }
                r_col *= shade;
                g_col *= shade;
                b_col *= shade;
            } else if (map_style == MC_STYLE_MC_MAP) {
                /* Vanilla Minecraft Filled Map paper contour & checkerboard water */
                float shade = 1.0f;
                if (ocean_like) {
                    int check = ((i + j) & 1);
                    if (d_c < -0.35f) {
                        shade = 0.72f;
                    } else if (d_c < -0.15f) {
                        shade = check ? 0.78f : 0.94f;
                    } else {
                        shade = check ? 0.92f : 1.06f;
                    }
                    r_col = r_col * 0.85f + 16.0f;
                    g_col = g_col * 0.92f + 16.0f;
                    b_col = b_col * 0.96f + 12.0f;
                } else {
                    float dz_n = depth[c] - depth[c - bw];
                    if (dz_n > 0.045f) shade = 1.15f;
                    else if (dz_n < -0.045f) shade = 0.80f;
                    else shade = ((i ^ j) & 1) ? 0.97f : 1.0f;

                    /* Warm vanilla parchment tint */
                    r_col = r_col * 0.86f + 28.0f;
                    g_col = g_col * 0.84f + 22.0f;
                    b_col = b_col * 0.78f + 14.0f;
                }
                r_col *= shade;
                g_col *= shade;
                b_col *= shade;
            }

            size_t out_i = ((size_t)j * (size_t)w + (size_t)i) * 4;
            out_rgba[out_i + 0] = clamp_u8(r_col);
            out_rgba[out_i + 1] = clamp_u8(g_col);
            out_rgba[out_i + 2] = clamp_u8(b_col);
            out_rgba[out_i + 3] = 255;
        }
    }

    free(ids);
    free(depth);
}

static void render_nether_tile(
    const McWorldHandle *h,
    int32_t scale,
    int32_t block_x0,
    int32_t block_z0,
    int32_t w,
    int32_t h_px,
    int32_t map_style,
    uint8_t *out_rgba
) {
    int32_t bw = w + 2;
    int32_t bh = h_px + 2;

    Range r;
    r.scale = scale;
    r.x = floor_div_i32(block_x0 - scale, scale);
    r.z = floor_div_i32(block_z0 - scale, scale);
    r.sx = bw;
    r.sz = bh;
    r.y = (scale == 1) ? 64 : 16; /* Y=64 lava lake horizon */
    r.sy = 1;
    /* allocCache reserves the temporary Voronoi source slab required at scale=1 */
    int32_t *ids = (int32_t *)allocCache(&h->g_net, r);
    if (!ids) return;
    genBiomes(&h->g_net, ids, r);

    for (int32_t j = 0; j < h_px; j++) {
        for (int32_t i = 0; i < w; i++) {
            int32_t ix = i + 1;
            int32_t jz = j + 1;
            size_t c = (size_t)jz * (size_t)bw + (size_t)ix;
            int id = ids[c];
            if (id < 0 || id >= 256) id = nether_wastes;

            float r_col = (float)h->colors[id][0];
            float g_col = (float)h->colors[id][1];
            float b_col = (float)h->colors[id][2];

            if (map_style != MC_STYLE_FLAT) {
                /* Subtle obsidian/magma grain + crisp biome outline */
                int bx = block_x0 + i * scale;
                int bz = block_z0 + j * scale;
                uint32_t hsh = (uint32_t)(bx * 73856093) ^ (uint32_t)(bz * 19349663);
                float grain = 0.94f + ((float)(hsh & 31) / 31.0f) * 0.12f;
                if (ids[c - 1] != id || ids[c + 1] != id || ids[c - bw] != id || ids[c + bw] != id) {
                    grain *= 0.80f;
                }
                r_col *= grain;
                g_col *= grain;
                b_col *= grain;
            }

            size_t out_i = ((size_t)j * (size_t)w + (size_t)i) * 4;
            out_rgba[out_i + 0] = clamp_u8(r_col);
            out_rgba[out_i + 1] = clamp_u8(g_col);
            out_rgba[out_i + 2] = clamp_u8(b_col);
            out_rgba[out_i + 3] = 255;
        }
    }

    free(ids);
}

static void render_end_tile(
    const McWorldHandle *h,
    int32_t scale,
    int32_t block_x0,
    int32_t block_z0,
    int32_t w,
    int32_t h_px,
    int32_t map_style,
    uint8_t *out_rgba
) {
    int32_t bw = w + 2;
    int32_t bh = h_px + 2;
    size_t count = (size_t)bw * (size_t)bh;
    float *y_map = NULL;

    Range r;
    r.scale = scale;
    r.x = floor_div_i32(block_x0 - scale, scale);
    r.z = floor_div_i32(block_z0 - scale, scale);
    r.sx = bw;
    r.sz = bh;
    r.y = (scale == 1) ? 64 : 16;
    r.sy = 1;
    int32_t *ids = (int32_t *)allocCache(&h->g_end, r);
    if (!ids) return;
    genBiomes(&h->g_end, ids, r);

    if (scale <= 8) {
        y_map = (float *)calloc(count, sizeof(float));
        if (y_map) {
            int32_t sx0 = floor_div_i32(block_x0 - scale, scale);
            int32_t sz0 = floor_div_i32(block_z0 - scale, scale);
            mapEndSurfaceHeight(y_map, &h->g_end.en, &h->sn_end, sx0, sz0, bw, bh, scale, 0);
            mapEndIslandHeight(y_map, &h->g_end.en, h->seed, sx0, sz0, bw, bh, scale);
        }
    }

    for (int32_t j = 0; j < h_px; j++) {
        for (int32_t i = 0; i < w; i++) {
            int32_t ix = i + 1;
            int32_t jz = j + 1;
            size_t c = (size_t)jz * (size_t)bw + (size_t)ix;
            int id = ids[c];
            if (id < 0 || id >= 256) id = the_end;

            int bx = block_x0 + i * scale;
            int bz = block_z0 + j * scale;
            double dist2 = (double)bx * (double)bx + (double)bz * (double)bz;

            float r_col, g_col, b_col;

            if (y_map) {
                float elev = y_map[c];
                if (elev <= 1.0f) {
                    /* Deep End Void with subtle cosmic tint */
                    r_col = 11.0f;
                    g_col = 8.0f;
                    b_col = 20.0f;
                    /* Inside 1200b moat highlight subtle concentric ring */
                    if (dist2 > 960000.0 && dist2 < 1040000.0) {
                        r_col += 8.0f; b_col += 14.0f;
                    }
                } else {
                    r_col = (float)h->colors[id][0];
                    g_col = (float)h->colors[id][1];
                    b_col = (float)h->colors[id][2];
                    if (id == small_end_islands) {
                        r_col = 178.0f; g_col = 170.0f; b_col = 122.0f;
                    }
                    if (map_style != MC_STYLE_FLAT) {
                        float dx = y_map[c + 1] - y_map[c - 1];
                        float dz = y_map[c + bw] - y_map[c - bw];
                        float shade = 0.86f + fminf(0.28f, (elev - 35.0f) * 0.008f) + (-dx - dz) * 0.025f;
                        if (shade < 0.60f) shade = 0.60f;
                        if (shade > 1.28f) shade = 1.28f;
                        r_col *= shade;
                        g_col *= shade;
                        b_col *= shade;
                    }
                }
            } else {
                /* Zoomed-out overview */
                if (id == small_end_islands) {
                    /* Void unless occasional outer island cluster noise */
                    r_col = 14.0f; g_col = 10.0f; b_col = 26.0f;
                } else if (id == the_end && dist2 > 28000.0 && dist2 < 1000000.0) {
                    /* Main Island surrounding Void Ring (~160 to 1000 blocks from 0,0) */
                    r_col = 12.0f; g_col = 9.0f; b_col = 22.0f;
                } else {
                    r_col = (float)h->colors[id][0];
                    g_col = (float)h->colors[id][1];
                    b_col = (float)h->colors[id][2];
                    if (map_style != MC_STYLE_FLAT) {
                        if (ids[c - 1] != id || ids[c + 1] != id || ids[c - bw] != id || ids[c + bw] != id) {
                            r_col *= 0.85f; g_col *= 0.85f; b_col *= 0.85f;
                        }
                    }
                }
            }

            size_t out_i = ((size_t)j * (size_t)w + (size_t)i) * 4;
            out_rgba[out_i + 0] = clamp_u8(r_col);
            out_rgba[out_i + 1] = clamp_u8(g_col);
            out_rgba[out_i + 2] = clamp_u8(b_col);
            out_rgba[out_i + 3] = 255;
        }
    }

    free(ids);
    if (y_map) free(y_map);
}

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
) {
    if (!h || !out_rgba || w <= 0 || h_px <= 0) return;
    if (scale != 1 && scale != 4 && scale != 16 && scale != 64 && scale != 256) scale = 16;

    if (dim == DIM_OVERWORLD) {
        render_overworld_tile(h, scale, block_x0, block_z0, w, h_px, map_style, out_rgba);
    } else if (dim == DIM_NETHER) {
        render_nether_tile(h, scale, block_x0, block_z0, w, h_px, map_style, out_rgba);
    } else {
        render_end_tile(h, scale, block_x0, block_z0, w, h_px, map_style, out_rgba);
    }
}

int32_t mc_sample_point(
    const McWorldHandle *h,
    int32_t dim,
    int32_t block_x,
    int32_t block_z,
    float *out_height,
    char *out_name,
    int32_t name_cap
) {
    if (!h) return -1;
    int32_t id = -1;
    float height = 64.0f;

    if (dim == DIM_OVERWORLD) {
        int64_t np[6];
        id = sampleBiomeNoise(&h->g_ow.bn, np, block_x >> 2, 16, block_z >> 2, NULL, 0);
        height = 64.0f + ((float)np[NP_DEPTH] / 10000.0f) * 128.0f;
    } else if (dim == DIM_NETHER) {
        id = getBiomeAt(&h->g_net, 4, block_x >> 2, 16, block_z >> 2);
        height = 64.0f;
    } else {
        id = getBiomeAt(&h->g_end, 4, block_x >> 2, 16, block_z >> 2);
        float y1 = 0.0f;
        mapEndSurfaceHeight(&y1, &h->g_end.en, &h->sn_end, block_x, block_z, 1, 1, 1, 0);
        mapEndIslandHeight(&y1, &h->g_end.en, h->seed, block_x, block_z, 1, 1, 1);
        height = y1;
    }

    if (out_height) *out_height = height;
    if (out_name && name_cap > 0) {
        const char *s = biome2str(h->mc_ver, id);
        if (!s) s = "unknown";
        strncpy(out_name, s, (size_t)(name_cap - 1));
        out_name[name_cap - 1] = '\0';
    }
    return id;
}

void mc_get_spawn(McWorldHandle *h, int32_t *out_x, int32_t *out_z) {
    if (!h) return;
    Pos sp = estimateSpawn(&h->g_ow, NULL);
    if (out_x) *out_x = sp.x;
    if (out_z) *out_z = sp.z;
}

int32_t mc_get_strongholds(McWorldHandle *h, McStructureHit *out, int32_t max_out) {
    if (!h || !out || max_out <= 0) return 0;
    StrongholdIter sh;
    initFirstStronghold(&sh, h->mc_ver, h->seed);
    int32_t count = 0;
    for (int32_t i = 0; i < 128 && count < max_out; i++) {
        if (nextStronghold(&sh, &h->g_ow) <= 0) break;
        out[count].stype = -1; /* Stronghold */
        out[count].x = sh.pos.x;
        out[count].z = sh.pos.z;
        out[count].variant = count + 1;
        out[count].biome = getBiomeAt(&h->g_ow, 4, sh.pos.x >> 2, 16, sh.pos.z >> 2);
        count++;
    }
    return count;
}

int32_t mc_get_end_gateways(McWorldHandle *h, McStructureHit *out_inner_20, McStructureHit *out_linked_20) {
    if (!h || !out_inner_20 || !out_linked_20) return 0;
    Pos inner[20];
    getFixedEndGateways(h->mc_ver, h->seed, inner);
    for (int i = 0; i < 20; i++) {
        Pos dst = getLinkedGatewayPos(&h->g_end.en, &h->sn_end, h->seed, inner[i]);
        out_inner_20[i].stype = End_Gateway;
        out_inner_20[i].x = inner[i].x;
        out_inner_20[i].z = inner[i].z;
        out_inner_20[i].variant = i + 1;
        out_inner_20[i].biome = the_end;

        out_linked_20[i].stype = End_Gateway;
        out_linked_20[i].x = dst.x;
        out_linked_20[i].z = dst.z;
        out_linked_20[i].variant = 101 + i;
        out_linked_20[i].biome = end_highlands;
    }
    return 20;
}

static void scan_one_structure_type(
    McWorldHandle *h,
    Generator *g,
    int32_t stype,
    int32_t min_bx,
    int32_t min_bz,
    int32_t max_bx,
    int32_t max_bz,
    McStructureHit *out,
    int32_t max_out,
    int32_t *count
) {
    StructureConfig sconf;
    if (!getStructureConfig(stype, h->mc_ver, &sconf)) return;

    int32_t reg_blocks = (int32_t)sconf.regionSize * 16;
    if (reg_blocks <= 0) return;

    int32_t rx0 = floor_div_i32(min_bx, reg_blocks) - 1;
    int32_t rz0 = floor_div_i32(min_bz, reg_blocks) - 1;
    int32_t rx1 = floor_div_i32(max_bx, reg_blocks) + 1;
    int32_t rz1 = floor_div_i32(max_bz, reg_blocks) + 1;

    /* Clamp absurd continent-wide loops to keep scanning real-time */
    if ((rx1 - rx0 + 1) * (rz1 - rz0 + 1) > 3600) {
        return;
    }

    Piece end_pieces[END_CITY_PIECES_MAX];

    for (int32_t rz = rz0; rz <= rz1 && *count < max_out; rz++) {
        for (int32_t rx = rx0; rx <= rx1 && *count < max_out; rx++) {
            Pos pos;
            if (!getStructurePos(stype, h->mc_ver, h->seed, rx, rz, &pos)) continue;
            if (pos.x < min_bx || pos.x > max_bx || pos.z < min_bz || pos.z > max_bz) continue;
            if (!isViableStructurePos(stype, g, pos.x, pos.z, 0)) continue;

            if (stype == End_City) {
                if (!isViableEndCityTerrain(g, &h->sn_end, pos.x, pos.z)) continue;
            } else if (g->dim == DIM_OVERWORLD) {
                if (!isViableStructureTerrain(stype, g, pos.x, pos.z)) continue;
            }

            int32_t variant = 0;
            int32_t biome = getBiomeAt(g, 4, pos.x >> 2, 16, pos.z >> 2);

            if (stype == End_City) {
                int npieces = getEndCityPieces(end_pieces, h->seed, pos.x >> 4, pos.z >> 4);
                for (int p = 0; p < npieces; p++) {
                    if (end_pieces[p].type == END_SHIP) {
                        variant |= 1; /* Has Elytra End Ship! */
                        break;
                    }
                }
            } else if (stype == Village || stype == Igloo || stype == Ruined_Portal || stype == Ruined_Portal_N) {
                StructureVariant sv;
                if (getVariant(&sv, stype, h->mc_ver, h->seed, pos.x, pos.z, biome)) {
                    if (sv.abandoned || sv.basement || sv.giant) variant |= 1;
                    if (sv.underground) variant |= 2;
                }
            }

            McStructureHit hit;
            hit.stype = stype;
            hit.x = pos.x;
            hit.z = pos.z;
            hit.variant = variant;
            hit.biome = biome;
            out[*count] = hit;
            (*count)++;
        }
    }
}

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
) {
    if (!h || !out || max_out <= 0) return 0;
    int32_t count = 0;

    if (dim == DIM_OVERWORLD) {
        /* Scan rarest / landmark structures first so common villages or portals never starve them */
        if (mask & MC_MASK_MANSION)
            scan_one_structure_type(h, &h->g_ow, Mansion, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
        if (mask & MC_MASK_MONUMENT)
            scan_one_structure_type(h, &h->g_ow, Monument, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
        if (mask & MC_MASK_ANCIENT_CITY)
            scan_one_structure_type(h, &h->g_ow, Ancient_City, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
        if (mask & MC_MASK_OUTPOST)
            scan_one_structure_type(h, &h->g_ow, Outpost, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
        if (mask & MC_MASK_JUNGLE_TEMPLE)
            scan_one_structure_type(h, &h->g_ow, Jungle_Temple, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
        if (mask & MC_MASK_SWAMP_HUT)
            scan_one_structure_type(h, &h->g_ow, Swamp_Hut, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
        if (mask & MC_MASK_DESERT_PYRAMID)
            scan_one_structure_type(h, &h->g_ow, Desert_Pyramid, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
        if (mask & MC_MASK_IGLOO)
            scan_one_structure_type(h, &h->g_ow, Igloo, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
        if (mask & MC_MASK_TRAIL_RUINS)
            scan_one_structure_type(h, &h->g_ow, Trail_Ruins, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
        if (mask & MC_MASK_TRIAL_CHAMBERS)
            scan_one_structure_type(h, &h->g_ow, Trial_Chambers, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
        if (mask & MC_MASK_RUINED_PORTAL)
            scan_one_structure_type(h, &h->g_ow, Ruined_Portal, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
        if (mask & MC_MASK_VILLAGE)
            scan_one_structure_type(h, &h->g_ow, Village, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
    } else if (dim == DIM_NETHER) {
        if (mask & MC_MASK_FORTRESS)
            scan_one_structure_type(h, &h->g_net, Fortress, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
        if (mask & MC_MASK_BASTION)
            scan_one_structure_type(h, &h->g_net, Bastion, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
        if (mask & MC_MASK_RUINED_PORTAL)
            scan_one_structure_type(h, &h->g_net, Ruined_Portal_N, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
    } else if (dim == DIM_END) {
        if (mask & MC_MASK_END_CITY)
            scan_one_structure_type(h, &h->g_end, End_City, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
        if (mask & MC_MASK_END_GATEWAY)
            scan_one_structure_type(h, &h->g_end, End_Gateway, min_bx, min_bz, max_bx, max_bz, out, max_out, &count);
    }

    return count;
}

static void insert_nearest_hit(
    McStructureHit *out,
    int64_t *dists,
    int32_t *count,
    int32_t max_out,
    McStructureHit hit,
    int64_t dist2
) {
    if (*count == max_out && dist2 >= dists[max_out - 1]) {
        return;
    }
    int32_t pos = *count;
    if (pos < max_out) {
        (*count)++;
    } else {
        pos = max_out - 1;
    }
    while (pos > 0 && dist2 < dists[pos - 1]) {
        out[pos] = out[pos - 1];
        dists[pos] = dists[pos - 1];
        pos--;
    }
    out[pos] = hit;
    dists[pos] = dist2;
}

int32_t mc_find_nearest_structures(
    McWorldHandle *h,
    int32_t dim,
    int32_t stype,
    int32_t only_end_ship,
    int32_t origin_x,
    int32_t origin_z,
    int32_t max_radius,
    McStructureHit *out,
    int32_t max_out
) {
    if (!h || !out || max_out <= 0) return 0;
    if (max_out > 64) max_out = 64;
    if (max_radius < 512) max_radius = 512;
    if (max_radius > 65536) max_radius = 65536;

    int64_t max_r2 = (int64_t)max_radius * (int64_t)max_radius;
    int64_t dists[64];
    int32_t count = 0;

    if (stype == -1) {
        /* Strongholds */
        McStructureHit all_sh[128];
        int32_t n = mc_get_strongholds(h, all_sh, 128);
        for (int32_t i = 0; i < n; i++) {
            int64_t dx = (int64_t)all_sh[i].x - (int64_t)origin_x;
            int64_t dz = (int64_t)all_sh[i].z - (int64_t)origin_z;
            int64_t d2 = dx * dx + dz * dz;
            if (d2 <= max_r2) {
                insert_nearest_hit(out, dists, &count, max_out, all_sh[i], d2);
            }
        }
        return count;
    }

    if (stype == End_Gateway && dim == DIM_END) {
        McStructureHit inner[20], linked[20];
        mc_get_end_gateways(h, inner, linked);
        for (int32_t i = 0; i < 20; i++) {
            int64_t dx = (int64_t)linked[i].x - (int64_t)origin_x;
            int64_t dz = (int64_t)linked[i].z - (int64_t)origin_z;
            int64_t d2 = dx * dx + dz * dz;
            if (d2 <= max_r2) {
                insert_nearest_hit(out, dists, &count, max_out, linked[i], d2);
            }
        }
        return count;
    }

    Generator *g = (dim == DIM_NETHER) ? &h->g_net : ((dim == DIM_END) ? &h->g_end : &h->g_ow);
    StructureConfig sconf;
    if (!getStructureConfig(stype, h->mc_ver, &sconf)) return 0;

    int32_t reg_blocks = (int32_t)sconf.regionSize * 16;
    if (reg_blocks <= 0) return 0;

    int32_t origin_rx = floor_div_i32(origin_x, reg_blocks);
    int32_t origin_rz = floor_div_i32(origin_z, reg_blocks);
    int32_t max_ring = (max_radius / reg_blocks) + 2;
    if (max_ring > 72) max_ring = 72;

    Piece end_pieces[END_CITY_PIECES_MAX];
    int32_t budget = 16000;

    for (int32_t ring = 0; ring <= max_ring && budget > 0; ring++) {
        if (count == max_out && ring > 1) {
            int64_t min_ring_dist = (int64_t)(ring - 1) * (int64_t)reg_blocks;
            if (min_ring_dist * min_ring_dist > dists[max_out - 1]) {
                break;
            }
        }

        int32_t rz0 = origin_rz - ring;
        int32_t rz1 = origin_rz + ring;
        int32_t rx0 = origin_rx - ring;
        int32_t rx1 = origin_rx + ring;

        for (int32_t rz = rz0; rz <= rz1 && budget > 0; rz++) {
            for (int32_t rx = rx0; rx <= rx1 && budget > 0; rx++) {
                if (ring > 0 && rz != rz0 && rz != rz1 && rx != rx0 && rx != rx1) {
                    continue;
                }
                budget--;

                Pos pos;
                if (!getStructurePos(stype, h->mc_ver, h->seed, rx, rz, &pos)) continue;

                int64_t dx = (int64_t)pos.x - (int64_t)origin_x;
                int64_t dz = (int64_t)pos.z - (int64_t)origin_z;
                int64_t d2 = dx * dx + dz * dz;
                if (d2 > max_r2) continue;
                if (count == max_out && d2 >= dists[max_out - 1]) continue;

                if (!isViableStructurePos(stype, g, pos.x, pos.z, 0)) continue;
                if (stype == End_City) {
                    if (!isViableEndCityTerrain(g, &h->sn_end, pos.x, pos.z)) continue;
                } else if (g->dim == DIM_OVERWORLD) {
                    if (!isViableStructureTerrain(stype, g, pos.x, pos.z)) continue;
                }

                int32_t variant = 0;
                int32_t biome = getBiomeAt(g, 4, pos.x >> 2, 16, pos.z >> 2);

                if (stype == End_City) {
                    int npieces = getEndCityPieces(end_pieces, h->seed, pos.x >> 4, pos.z >> 4);
                    for (int p = 0; p < npieces; p++) {
                        if (end_pieces[p].type == END_SHIP) {
                            variant |= 1;
                            break;
                        }
                    }
                    if (only_end_ship && !(variant & 1)) {
                        continue;
                    }
                } else if (stype == Village || stype == Igloo || stype == Ruined_Portal || stype == Ruined_Portal_N) {
                    StructureVariant sv;
                    if (getVariant(&sv, stype, h->mc_ver, h->seed, pos.x, pos.z, biome)) {
                        if (sv.abandoned || sv.basement || sv.giant) variant |= 1;
                        if (sv.underground) variant |= 2;
                    }
                }

                McStructureHit hit;
                hit.stype = stype;
                hit.x = pos.x;
                hit.z = pos.z;
                hit.variant = variant;
                hit.biome = biome;
                insert_nearest_hit(out, dists, &count, max_out, hit, d2);
            }
        }
    }

    return count;
}

int32_t mc_find_nearest_biomes(
    const McWorldHandle *h,
    int32_t dim,
    int32_t target_biome_id,
    int32_t origin_x,
    int32_t origin_z,
    int32_t max_radius,
    McBiomeHit *out,
    int32_t max_out
) {
    if (!h || !out || max_out <= 0) return 0;
    if (!biomeExists(h->mc_ver, target_biome_id)) return 0;
    if (max_out > 32) max_out = 32;
    if (max_radius < 512) max_radius = 512;
    if (max_radius > 24576) max_radius = 24576;

    const Generator *g = (dim == DIM_NETHER) ? &h->g_net : ((dim == DIM_END) ? &h->g_end : &h->g_ow);
    int32_t y_q = 16; /* Y=64 in 1:4 scale */
    if (target_biome_id == deep_dark) {
        y_q = -12; /* Y=-48 */
    } else if (target_biome_id == lush_caves || target_biome_id == dripstone_caves) {
        y_q = 4; /* Y=16 */
    }

    int64_t dists[32];
    int32_t count = 0;
    const int64_t min_sep2 = 384LL * 384LL;
    int32_t budget = 90000;

    /* Progressive concentric square rings with adaptive step for fast discovery */
    for (int32_t r = 0; r <= max_radius && budget > 0; ) {
        int32_t step = (r < 4096) ? 64 : ((r < 12288) ? 96 : 160);
        if (count == max_out && r > step) {
            int64_t min_r2 = (int64_t)(r - step) * (int64_t)(r - step);
            if (min_r2 > dists[max_out - 1]) {
                break;
            }
        }

        if (r == 0) {
            int32_t bid = getBiomeAt(g, 4, origin_x >> 2, y_q, origin_z >> 2);
            budget--;
            if (bid == target_biome_id) {
                out[0].biome = bid;
                out[0].x = origin_x;
                out[0].z = origin_z;
                dists[0] = 0;
                count = 1;
            }
            r += step;
            continue;
        }

        /* Walk the 4 perimeter edges of square [-r..r] */
        for (int32_t side = 0; side < 4 && budget > 0; side++) {
            for (int32_t t = -r; t < r && budget > 0; t += step) {
                int32_t dx_b = 0, dz_b = 0;
                if (side == 0)      { dx_b = t;  dz_b = -r; }
                else if (side == 1) { dx_b = r;  dz_b = t;  }
                else if (side == 2) { dx_b = -t; dz_b = r;  }
                else                { dx_b = -r; dz_b = -t; }

                int32_t bx = origin_x + dx_b;
                int32_t bz = origin_z + dz_b;
                budget--;

                int32_t bid = getBiomeAt(g, 4, bx >> 2, y_q, bz >> 2);
                if (bid != target_biome_id) continue;

                int64_t d2 = (int64_t)dx_b * (int64_t)dx_b + (int64_t)dz_b * (int64_t)dz_b;
                if (count == max_out && d2 >= dists[max_out - 1]) continue;

                /* Deduplicate nearby samples belonging to the same biome patch */
                int duplicate = 0;
                for (int32_t k = 0; k < count; k++) {
                    int64_t sx = (int64_t)out[k].x - (int64_t)bx;
                    int64_t sz = (int64_t)out[k].z - (int64_t)bz;
                    if (sx * sx + sz * sz < min_sep2) {
                        duplicate = 1;
                        break;
                    }
                }
                if (duplicate) continue;

                int32_t pos = count;
                if (pos < max_out) count++;
                else pos = max_out - 1;
                while (pos > 0 && d2 < dists[pos - 1]) {
                    out[pos] = out[pos - 1];
                    dists[pos] = dists[pos - 1];
                    pos--;
                }
                out[pos].biome = bid;
                out[pos].x = bx;
                out[pos].z = bz;
                dists[pos] = d2;
            }
        }

        r += step;
    }

    return count;
}
