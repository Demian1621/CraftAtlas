# CraftAtlas — Release Notes (`v1.2.0`)

> **[Español](#novedades-en-español) · [English](#whats-new-in-english)**

---

## Novedades en Español

### 🎯 Precisión Real Multi-Versión en Toda la App (`1.18.2` → `26.3`)
- **Motor Cubiomes + Capas + Buscador Radial por Versión:** Ahora `Java 1.18.2`, `1.19.4`, `1.20.6`, `1.21.1`, `1.21.4` y `26.3` adaptan tanto la generación de biomas/estructuras como las capas visibles y objetivos de búsqueda (sin *Trial Chambers* antes de `1.21.1`, sin *Trail Ruins/Cherry Grove* antes de `1.20.6` ni *Ancient Cities/Manglar* en `1.18.2`).
- **Catálogo JEI / EMI + Alquimia fieles por versión (`2,034` recetas):** Oculta automáticamente bloques/herramientas futuros en versiones anteriores e incorpora recetas históricas exactas: **Herrería Netherita de 2 slots sin plantilla en `1.18.2–1.19.4`**, Magnetita con Lingote de Netherita y Rienda con Slime hasta `1.21.4`, y tabla de Alquimia de 15 vs 19 pociones.
- **Libros Encantados, Equipamiento God, Granjas, Glitches y Notas:** Los libros (`38` en `1.18.2` → `43` en `26.3`), piezas God (*Pantalones* sin *Sigilo Rápido* en `1.18.2`), granjas y dupes muestran el estado exacto de tu versión (*String Duper* activo hasta `1.21.1` / parcheado desde `1.21.2`, *Void Trading* activo hasta `1.20.6`, *StackOverflow* nativo en `1.18.2`, *Curación Zombi x5* hasta `1.19.4`, *Stacking Raid Farm* pre-`1.21`).

### ⚔️ Nueva Vista "God Armor (Ilegal / Glitches)"
- **Sets con las 4 Protecciones IV simultáneas** (Casco, Pechera, Pantalones y Botas con tope real **EPF 20 = 80% de reducción**, toggle interactivo de Espinas III y *Sigilo Rápido III* desde `1.19.4+`), Arco *Infinidad + Reparación*, Ballesta *Multidisparo + Perforación*, Espada triple daño, Pico *Fortuna + Toque de Seda* y Mazo *Densidad + Fisura* (`1.21.1+`).
- Incluye guía **Survival Vanilla (`1.14.0–1.14.2` hacia adelante)** con advertencia crítica anti-downgrade, **comandos `/give` exactos adaptados a cada era** (`{Enchantments:[...]}` vs `levels:{}` vs `26.3`) con copia en 1 clic, y plantilla **Datapack JSON (`exclusive_set/armor.json`)**.

### 🎬 4 Temas de Video Animados + Importador `.mp4` / `.webm` / `.gif`
- Nuevo motor streaming nativo `.cavf` en Rust (`0` dependencias externas nuevas, textura GPU única reciclada, pausa automática al `0%` CPU sin foco) con **4 fondos de video integrados** (*Ciudad Antigua Sculk*, *Trial Chambers*, *Jardín Pálido*, *Monumento Oceánico*) e **importador de video propio** validado con límites estrictos.

### 📦 Lector `.litematic` + Lista de Compras + Updater SHA-256 + Modularización
- Incluye lector nativo de esquemáticos `.litematic` (con *drag-and-drop*), Lista de Compras Unificada con cálculo en Shulkers (`x1,728`), Auto-Updater in-app *fail-closed* con SHA-256 (FIPS 180-4) y modularización limpia de `seed_map/` (`layers.rs`, `worlds.rs`), `recipes/`, `right_rail.rs` y `updater_modal.rs`.

---

## What's New in English

### 🎯 True App-Wide Multi-Version Accuracy (`1.18.2` → `26.3`)
- **Version-Exact Cubiomes Worldgen, Layers & Finder:** Switching versions (`1.18.2`, `1.19.4`, `1.20.6`, `1.21.1`, `1.21.4`, `26.3`) adapts both C terrain/structure generation and UI layers/search targets so unavailable structures and biomes never clutter older versions.
- **JEI Crafting Trees & Alchemy (`2,034` Recipes):** Excludes post-release items/ingredients automatically and switches to historical recipes (**2-slot template-free Netherite smithing in `1.18.2–1.19.4`**, Netherite Lodestone & Slimeball Lead pre-`26.3`, and 15 vs 19 brewing rows).
- **Enchant Books, God Gear, Farms & Glitches:** Enchantment books (`38` in `1.18.2` up to `43` in `26.3`), Leggings pre-Swift Sneak (`1.18.2`), Raid/Witch/Bamboo farm overrides, and version-reactive Glitch statuses (*String Duper*, *Void Trading*, *StackOverflow*, *5x Villager Curing*) update dynamically with expanded per-version notes.

### ⚔️ New "God Armor (Illegal / Glitches)" Sub-View
- Covers **All-4-Protection Armor** (`EPF 20 / 80% cap` math + Thorns III toggle + version-aware `Swift Sneak III`), *Mending + Infinity* Bow (`1.9–1.11.1`), *Multishot + Piercing* Crossbow, triple-damage Sword, *Fortune + Silk Touch* Pickaxe, and *Density + Breach* Mace (`1.21.1+`).
- Features step-by-step **Vanilla `1.14.2` forward-only Survival instructions** (with strict anti-downgrade corruption warning), **version-exact `/give` commands** (NBT vs `levels:{}` vs `26.3` simplified syntax), and **vanilla Datapack JSON (`#exclusive_set`)**.

### 🎬 4 Embedded Animated Video Themes + Custom Video Importer
- Native Rust `.cavf` streaming player (zero new crates, single in-place GPU texture, `0%` CPU pause when unfocused) bundling 4 animated themes (*Deep Dark*, *Trial Chambers*, *Pale Garden*, *Ocean Monument*) plus safe `.mp4` / `.webm` / `.mov` / `.mkv` / `.gif` custom wallpaper importing.

### 📦 `.litematic` Parser, Unified Shopping List, SHA-256 Updater & Refactor
- Includes native `.litematic` decoding + drag-and-drop, persistent Shulker-aware Shopping List, pure-Rust FIPS 180-4 SHA-256 fail-closed auto-updater with Build Provenance attestations, and modular architecture across `seed_map/`, `recipes/`, `right_rail.rs`, and `updater_modal.rs`.
