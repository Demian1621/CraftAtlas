<div align="center">

# CraftAtlas (`v1.2.0`)

**The native desktop co-pilot for technical survival in Minecraft Java (`1.18.2` – `26.3`) — built in Rust + C, 100% offline-ready, cross-platform across Windows, macOS, and Linux.**

[![CI](https://github.com/Demian1621/CraftAtlas/actions/workflows/ci.yml/badge.svg)](https://github.com/Demian1621/CraftAtlas/actions/workflows/ci.yml)
[![Release](https://github.com/Demian1621/CraftAtlas/actions/workflows/release.yml/badge.svg)](https://github.com/Demian1621/CraftAtlas/actions/workflows/release.yml)

[English](#-built-for-players-tired-of-14-browser-tabs) · [Versión en Español](README.md) · [Security](SECURITY.md) · [Releases](https://github.com/Demian1621/CraftAtlas/releases) · [Feedback](https://github.com/Demian1621/CraftAtlas/issues/new/choose) · [Support on Ko-fi](https://ko-fi.com/titusdb)

</div>

---

## 🎯 Built for players tired of 14 browser tabs

If you play technical survival or min-max a long-term Java world, you know the drill: one browser tab open for Chunkbase to find a Woodland Mansion, another tab for the wiki's anvil order so your boots don't lock behind *"Too Expensive!"*, another calculator for 8:1 Nether portal coordinates, JEI open in-game just to check a recipe chain, and scattered notes for farm block counts.

**CraftAtlas** *(originally released as Minecraft Companion in `v1.0.0`–`v1.0.2`)* puts all of that into a **single native desktop binary** written in **Rust (`eframe`/`egui`)** powered by the native **C `cubiomes` engine**. No Electron bloat, no ads, instant launch, and automatic detection of your local `.minecraft/saves` worlds.

---

## 🧭 What's inside v1.2.0

### 1. 🗺️ Multi-Version Native Seed Map Explorer (`cubiomes` Engine · `1.18.2` – `26.3`)
- **True Multi-Version Support (`1.18.2`, `1.19.4`, `1.20.6`, `1.21.1`, `1.21.4`, `26.3`):** Switch Java versions in the top bar to match real biome, stronghold, and structure generation, automatically hiding future structures/biomes from Layers and Radial Search in older releases.
- **Radial Nearest Structure & Biome Finder:** Non-blocking C-accelerated search for the 10 nearest structures or biomes (Woodland Mansion, Ancient City, Stronghold, Ocean Monument, Trial Chambers, Elytra End Ship, Mushroom Fields, Cherry Grove, Pale Garden, Deep Dark, Lush Caves, etc.) sorted by exact block distance with one-click **`Go`** jump and `/tp` copy.
- **Overworld, Nether & The End** with high-speed biome rendering, 3D-shaded topographic terrain style, chunk grid, and **Slime Chunk** overlays.
- **Automatic Local World Detection**: scans Official `.minecraft/saves`, macOS `Application Support/minecraft`, Linux `~/.minecraft`, Flatpak (`com.mojang.Minecraft`), and **PrismLauncher** instances (sorted by last played) to pull your seed from `level.dat` with one click (text seeds and `Large Biomes` supported too).
- **Saved Seeds & History (Server-Friendly):** Bookmark named seeds (ideal for online SMP servers missing from `.minecraft/saves`) for one-click switching and browse an automatic history of your last 20 visited seeds.
- **Last Saved Player Location & Custom Skin (`.png`)**: displays your player's last saved `level.dat` position (`Player.Pos` + dimension) clearly labeled as non-live, with one-click reload and `.png` skin upload/drag-and-drop (`64x64`, `64x32`, or HD) to render your real skin face on the map.
- **Rare-First Structure Discovery**: Strongholds, **Woodland Mansions** (including Java 26.3's `pale_garden` + `dark_forest` rules), Ocean Monuments, **Ancient Cities (Warden head icon)**, Trial Chambers, Villages, Outposts, Bastions, Nether Fortresses, and End Cities with Elytra Ships.
- **Synced Overworld ↔ Nether (8:1) navigation**, interactive **Ruler** measurement tool, persistent **Waypoints** (with classic red bed icon for `Home / Bed`), right-click **temporary pins with instant `/tp` clipboard copy**, and **PNG map export**.

### 2. ⚒️ God Gear, Illegal God Armor & Enchanted Books (29 Presets · 43 Books)
- Curated end-game setups strictly filtered per active Java version (omitting *Swift Sneak* in `1.18.2`, *Brush/Wolf Armor* pre-`1.20.6`, and *Mace* pre-`1.21.1`).
- **New `God Armor (Illegal / Glitches)` Sub-View:** Complete guide to crafting illegal **all-4-Protections armor** (reaching the **20 EPF / 80% cap**), *Mending + Infinity* Bow (`1.9–1.11.1`), triple-damage Sword, dual-enchant Crossbow, and *Density + Breach* Mace (`1.21.1+`) — featuring `1.14.2` forward-only Survival steps (with anti-downgrade corruption warning), optimal binary Anvil merge pyramid, **version-exact `/give` commands**, and **Datapack JSON (`#exclusive_set`)**.
- **Version-Aware Enchanted Books Guide**: filters all **43 official Minecraft Java enchantments** and compatible item icons per version (`38` in `1.18.2` up to `43` in `26.3`), sources, incompatibilities, and Villager-excluded alerts (*Swift Sneak*, *Soul Speed*, *Wind Burst*).
- **True Java Edition Anvil Tree Optimizer**: computes optimal binary merge order, tracks **Prior Work Penalty**, and keeps anvil costs below the **39-level "Too Expensive!" cap**.

### 3. 📖 JEI / EMI Recipe Browser & Full Alchemy Guide (2,034 Recipes)
- Version-exact **`Recipes For` / `Uses For`** lookup and **recursive Crafting Tree**: switches automatically to **legacy 2-slot Netherite Smithing (no template)** in `1.18.2–1.19.4`, Netherite Lodestone & Slimeball Lead prior to `26.3`, and excludes future items/potions in older versions.
- Built-in **Brewing & Potions Quick-Reference** adapting dynamically (`15` core potions pre-1.21, `19` with Trial Potions in `1.21.1+`).

### 4. 🌾 Technical Farms, Native `.litematic` Reader & Shopping List (113 Farms · 31 Creators)
- Version-aware catalog hiding post-release farms (*Crafter/Trial/Resin/Armadillo*) on older Java versions while overriding *Stacking Raid Farms*, *Witch Huts*, and *Bamboo Farms* accurately per version.
- **Native `.litematic` Schematic Parser:** Click local `.litematic` files in `.minecraft/schematics`, pick a file, or **drag-and-drop onto the window** to inspect decoded block totals (including double slabs) in **Shulker Boxes (`x1,728`) + Stacks (`x64`)**.
- **Unified Persistent Shopping List:** Send materials in 1 click from **Farms**, **JEI Craft Trees**, or **`.litematic` schematics** to the right-side Shopping panel with progress tracking and Shulker breakdown.
- Smart version- & language-aware tutorial search paired with a curated roster of **31 Spanish, English, and Chinese technical Minecraft creators**, featuring real Minecraft skin avatars and specialty tags (`Farms`, `Dupes/Exploits`, `Redstone`).

### 5. ⚡ Dupes, Glitches & Portal Sync Calculator (34 Entries)
- Version-reactive status cards (*String Duper* working in `1.18.2–1.21.1` vs patched `1.21.4+`; *Void Trading* working pre-`1.21`; *StackOverflow Suppression* native in `1.18.2` vs CCE/Light in `1.19.4+`; *5x Zombie Cure* pre-`1.20.2` vs 1x cap), plus the **Overworld ↔ Nether Portal Sync Calculator**.

### 6. 💡 Pro Tips & Animated Video Themes (17 Guides · 10 Themes + Custom Video · 3 Locales)
- **2,308 pixel-crisp 32x32 item/mob icons**, procedural desktop-safe SFX audio, **10 built-in themes** including **4 embedded looping `.cavf` video backdrops** (*Deep Dark*, *Trial Chambers*, *Pale Garden*, *Ocean Monument*) + **Custom Video Importer (`.mp4`/`.webm`/`.gif`)** with low-RAM background decoding and `0%` idle CPU when unfocused, universal **`Ctrl + F` Command Palette**, pinned **Favorites rail**, **`EN-US` / `ES-MX` / `ES-ES`**, and **SHA-256 verified auto-updater**.

---

## 💻 Download & Install (Windows · macOS · Linux)

Grab the latest build for your platform on **[Releases](https://github.com/Demian1621/CraftAtlas/releases)**:

### 🪟 Windows (x86_64)
- Download the portable **`.exe`** (single executable, run directly) or the **`.zip`** bundle.

### 🍎 macOS (Universal Binary: Apple Silicon M1–M4 + Intel)
1. Download **`craft-atlas-v1.2.0-macos-universal.dmg`**.
2. Drag **CraftAtlas.app** into **Applications**.
3. **First launch note (Gatekeeper)**: Since this build uses an ad-hoc signature without a paid Apple Developer certificate, right-click (**Control + Click → Open → Open**) on the first launch, or run once in Terminal:
   ```bash
   xattr -cr "/Applications/CraftAtlas.app"
   ```

### 🐧 Linux (Multi-Distro)
| Package | Target Distros | Install / Run |
| --- | --- | --- |
| **`.AppImage`** *(Universal)* | Ubuntu, Fedora, Arch, Mint, Pop!_OS, openSUSE, SteamOS | `chmod +x craft-atlas*.AppImage && ./craft-atlas*.AppImage` |
| **`.deb`** | Ubuntu, Debian, Linux Mint, Pop!_OS | `sudo apt install ./craft-atlas_1.2.0_amd64.deb` |
| **`.rpm`** | Fedora, RHEL, Rocky Linux, openSUSE | `sudo dnf install ./craft-atlas-1.2.0-1.x86_64.rpm` |
| **`.tar.gz`** *(Portable)* | Arch Linux, Manjaro, NixOS, generic | Extract and run `./craft-atlas` or `./install-user.sh` |

> **Checksum & Build Provenance**: Every GitHub Release includes `SHA256SUMS.txt` and SLSA/Sigstore Build Provenance Attestations (see [SECURITY.md](SECURITY.md)).

---

## 🛠️ Building from Source

Requires **stable Rust (`cargo`)** and a C compiler (`gcc`, `clang`, or MSVC) for `cubiomes`:

```bash
# Ubuntu / Debian build dependencies:
sudo apt-get install -y build-essential pkg-config libasound2-dev \
  libxkbcommon-dev libwayland-dev libgl1-mesa-dev libxcursor-dev \
  libxrandr-dev libxi-dev libx11-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev

cargo test --locked
cargo build --release --locked
```

---

## ⚡ What's New in `v1.2.0`

- **True App-Wide Version Accuracy (`1.18.2` → `26.3`):** Switching Java version now updates Seed Map worldgen & Radial Finder, JEI Crafting (2-slot legacy Netherite, Netherite Lodestone, Slimeball Lead), Brewing, Enchant Books, God Gear, Farms, Glitches, and version release notes.
- **Illegal God Armor & Legacy Glitch Guide:** Step-by-step `1.14.2` forward-only Survival paths (80% EPF cap math), `/give` commands adapted per version syntax, and `1.21+` Datapack `exclusive_set` overrides.
- **4 Embedded `.cavf` Animated Video Themes + Custom `.mp4` Importer:** Zero-new-crate streaming playback (`0%` idle CPU) for *Deep Dark*, *Trial Chambers*, *Pale Garden*, and *Ocean Monument*, plus custom user video wall support.
- **`.litematic` Parser, Unified Shopping List & Fail-Closed SHA-256 Auto-Updater:** Full notes in [RELEASE_v1.2.0.md](docs/RELEASE_v1.2.0.md).

---

## 💬 Feedback & Supporting the Project

CraftAtlas is **100% free, ad-free, and has zero paywalled features**. Everything is unlocked for everyone.

- **Found a bug or have an idea?** Use [GitHub Issues](https://github.com/Demian1621/CraftAtlas/issues/new/choose) or join [GitHub Discussions](https://github.com/Demian1621/CraftAtlas/discussions).
- **Want to buy me a coffee? (Optional):** You can support ongoing development on [Ko-fi (`https://ko-fi.com/titusdb`)](https://ko-fi.com/titusdb) or via the **Support** button in the app's top bar.

---

## 🚀 Verified Updates & Security

`v1.2.0` ships with a **fail-closed in-app auto-updater verified with pure-Rust FIPS 180-4 SHA-256** over TLS 1.2+, paired with **GitHub Build Provenance Attestations** on every release artifact.

---

<div align="center">
Crafted with Rust, C, and technical survival passion · <b>Demian1621</b>
</div>
