<div align="center">

# CraftAtlas (`v1.1.0`)

**The native desktop co-pilot for technical survival in Minecraft Java (`26.3`) — built in Rust + C, 100% offline-ready, cross-platform across Windows, macOS, and Linux.**

[![CI](https://github.com/Demian1621/CraftAtlas/actions/workflows/ci.yml/badge.svg)](https://github.com/Demian1621/CraftAtlas/actions/workflows/ci.yml)
[![Release](https://github.com/Demian1621/CraftAtlas/actions/workflows/release.yml/badge.svg)](https://github.com/Demian1621/CraftAtlas/actions/workflows/release.yml)

[English](#-built-for-players-tired-of-14-browser-tabs) · [Versión en Español](README.md) · [Security](SECURITY.md) · [Releases](https://github.com/Demian1621/CraftAtlas/releases) · [Feedback](https://github.com/Demian1621/CraftAtlas/issues/new/choose) · [Support on Ko-fi](https://ko-fi.com/titusdb)

</div>

---

## 🎯 Built for players tired of 14 browser tabs

If you play technical survival or min-max a long-term Java world, you know the drill: one browser tab open for Chunkbase to find a Woodland Mansion, another tab for the wiki's anvil order so your boots don't lock behind *"Too Expensive!"*, another calculator for 8:1 Nether portal coordinates, JEI open in-game just to check a recipe chain, and scattered notes for farm block counts.

**CraftAtlas** *(originally released as Minecraft Companion in `v1.0.0`–`v1.0.2`)* puts all of that into a **single native desktop binary** written in **Rust (`eframe`/`egui`)** powered by the native **C `cubiomes` engine**. No Electron bloat, no ads, instant launch, and automatic detection of your local `.minecraft/saves` worlds.

---

## 🧭 What's inside v1.1.0

### 1. 🗺️ Native Seed Map Explorer (`cubiomes` Engine)
- **Overworld, Nether & The End** with high-speed biome rendering, 3D-shaded topographic terrain style, chunk grid, and **Slime Chunk** overlays.
- **Automatic Local World Detection**: scans Official `.minecraft/saves`, macOS `Application Support/minecraft`, Linux `~/.minecraft`, Flatpak (`com.mojang.Minecraft`), and **PrismLauncher** instances (sorted by last played) to pull your seed from `level.dat` with one click (text seeds and `Large Biomes` supported too).
- **Saved Seeds & History (Server-Friendly):** Bookmark named seeds (ideal for online SMP servers missing from `.minecraft/saves`) for one-click switching and browse an automatic history of your last 20 visited seeds.
- **Last Saved Player Location & Custom Skin (`.png`)**: displays your player's last saved `level.dat` position (`Player.Pos` + dimension) clearly labeled as non-live, with one-click reload and `.png` skin upload/drag-and-drop (`64x64`, `64x32`, or HD) to render your real skin face on the map.
- **Rare-First Structure Discovery**: Strongholds, **Woodland Mansions** (including Java 26.3's `pale_garden` + `dark_forest` rules), Ocean Monuments, **Ancient Cities (Warden head icon)**, Trial Chambers, Villages, Outposts, Bastions, Nether Fortresses, and End Cities with Elytra Ships.
- **Synced Overworld ↔ Nether (8:1) navigation**, interactive **Ruler** measurement tool, persistent **Waypoints** (with classic red bed icon for `Home / Bed`), right-click **temporary pins with instant `/tp` clipboard copy**, and **PNG map export**.

### 2. ⚒️ God Gear, Living Anvil & Enchanted Books (29 Presets · 43 Books)
- Curated end-game setups for armor, tools, weapons (including **Spear / Lunge**), bow/crossbow/trident/mace, and elytra — explaining what every enchantment does and flagging mutual incompatibilities.
- **New Switched Enchanted Books Guide**: covers all **43 official Minecraft Java (`26.3`) enchantments**, exact sources (Enchanting Table, Librarian Villager, Fishing, Piglin Bartering, Chests/Vaults), incompatibilities, optimal acquisition strategy, and a dedicated filter/warning for **books Librarian Villagers NEVER sell** (*Swift Sneak*, *Soul Speed*, *Wind Burst*).
- **True Java Edition Anvil Tree Optimizer**: computes optimal binary merge order vs. naive sequential stacking, tracks **Prior Work Penalty** accurately, saves levels, and guarantees you stay below the **39-level "Too Expensive!" cap**.
- Includes the interactive **Living Anvil** forge step-through simulator.

### 3. 📖 JEI / EMI Recipe Browser & Full Alchemy Guide (2,023 Recipes)
- Instant **`Recipes For` / `Uses For`** lookup with back/forward history, tag cycling, Crafting Table, Smelting, Blasting, Smoking, Smithing, Stonecutting, and **recursive Crafting Tree + raw material calculator**.
- Built-in **Brewing & Potions Quick-Reference**: Awkward base, all 16 core potion effects, Redstone (`+Duration`), Glowstone (`+Level II`), Gunpowder (`Splash`), Dragon's Breath (`Lingering`), Fermented Spider Eye (`Corruption`), and Tipped Arrows.

### 4. 🌾 Technical Farms & Curated Creators (113 Farms · 31 Creators)
- Filter farms by progression tier (`Early`, `Mid`, `End Game`) and resource type, with persistent checklists switchable between **Stacks (x64)** and **exact counts**.
- Scans local **Litematica (`.litematic` / `.schematic`)** folders on your machine.
- Smart version- & language-aware tutorial search paired with a curated roster of **31 Spanish, English, and Chinese technical Minecraft creators**, featuring real Minecraft skin avatars and specialty tags (`Farms`, `Dupes/Exploits`, `Redstone`).

### 5. ⚡ Dupes, Glitches & Portal Sync Calculator (34 Entries)
- Technical breakdown distinguishing **working 26.3 mechanics** from patched/historical exploits, required components, mechanics explanation, and a built-in **Overworld ↔ Nether Portal Sync Calculator**.

### 6. 💡 Pro Tips & Polished Desktop UX (17 Guides · 6 Themes · 3 Locales)
- **2,308 pixel-crisp 32x32 item/mob icons** packed into an embedded atlas, procedural Web/desktop-safe SFX audio (toggleable), **6 animated panoramic/scenic themes**, universal **`Ctrl + F` Command Palette**, pinned **Favorites rail**, **`EN-US` / `ES-MX` / `ES-ES`** localization, and a **non-intrusive GitHub Releases update checker** built right into the title bar.

---

## 💻 Download & Install (Windows · macOS · Linux)

Grab the latest build for your platform on **[Releases](https://github.com/Demian1621/CraftAtlas/releases)**:

### 🪟 Windows (x86_64)
- Download the portable **`.exe`** (single executable, run directly) or the **`.zip`** bundle.

### 🍎 macOS (Universal Binary: Apple Silicon M1–M4 + Intel)
1. Download **`craft-atlas-v1.1.0-macos-universal.dmg`**.
2. Drag **CraftAtlas.app** into **Applications**.
3. **First launch note (Gatekeeper)**: Since this build uses an ad-hoc signature without a paid Apple Developer certificate, right-click (**Control + Click → Open → Open**) on the first launch, or run once in Terminal:
   ```bash
   xattr -cr "/Applications/CraftAtlas.app"
   ```

### 🐧 Linux (Multi-Distro)
| Package | Target Distros | Install / Run |
| --- | --- | --- |
| **`.AppImage`** *(Universal)* | Ubuntu, Fedora, Arch, Mint, Pop!_OS, openSUSE, SteamOS | `chmod +x craft-atlas*.AppImage && ./craft-atlas*.AppImage` |
| **`.deb`** | Ubuntu, Debian, Linux Mint, Pop!_OS | `sudo apt install ./craft-atlas_1.1.0_amd64.deb` |
| **`.rpm`** | Fedora, RHEL, Rocky Linux, openSUSE | `sudo dnf install ./craft-atlas-1.1.0-1.x86_64.rpm` |
| **`.tar.gz`** *(Portable)* | Arch Linux, Manjaro, NixOS, generic | Extract and run `./craft-atlas` or `./install-user.sh` |

> **Checksum verification**: Every GitHub Release includes `SHA256SUMS.txt` so you can verify binaries cleanly (see [SECURITY.md](SECURITY.md)).

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

## ⚡ What's New in `v1.1.0`

- **Saved Server Seeds, History, Last Position & `.png` Skin:** Bookmark custom-named seeds (great for online servers lacking local `level.dat`), revisit your last 20 explored seeds from history, view `Player.Pos` (labeled non-live), and upload or drag-and-drop your `.png` skin.
- **Complete Enchanted Books Guide (43 Java `26.3` Enchantments):** Switched sub-view inside **God Gear & Anvil** pairing the Enchanted Book icon with compatible gear icons, acquisition channels, incompatibilities, optimal strategy, and a prominent filter for books villagers cannot trade (*Swift Sneak*, *Soul Speed*, *Wind Burst*).
- **Refreshed Seed Map Layout & Icons:** Clean 3-tab sidebar (**Waypoints** first at top, 2-column **Layers** tiles, and **Worlds / Skin**) with a unified top-bar world/server selector, classic isometric red bed for `Home / Bed`, and a pixel-art Warden head for `Ancient City`.
- Keeps all `v1.0.2` efficiency updates (**~0% unfocused CPU usage**, **30 FPS active cap**, Feedback/Ko-fi links, and balanced anvil SFX). Full notes in [RELEASE_v1.1.0.md](docs/RELEASE_v1.1.0.md).

---

## 💬 Feedback & Supporting the Project

CraftAtlas is **100% free, ad-free, and has zero paywalled features**. Everything is unlocked for everyone.

- **Found a bug or have an idea?** Use [GitHub Issues](https://github.com/Demian1621/CraftAtlas/issues/new/choose) or join [GitHub Discussions](https://github.com/Demian1621/CraftAtlas/discussions).
- **Want to buy me a coffee? (Optional):** You can support ongoing development on [Ko-fi (`https://ko-fi.com/titusdb`)](https://ko-fi.com/titusdb) or via the **Support** button in the app's top bar.

---

## 🚀 Future Roadmap

`v1.1.0` ships with TLS-hardened background release detection against **GitHub Releases** (and parses release asset SHA-256 digests). Upcoming `v1.2+` releases will add **verified in-app binary auto-update installation** alongside expanded Minecraft version support.

---

<div align="center">
Crafted with Rust, C, and technical survival passion · <b>Demian1621</b>
</div>
