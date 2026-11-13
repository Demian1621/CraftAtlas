# Minecraft Companion `v1.0.0`

**Primera versión pública multiplataforma** para **Windows (x64)**, **macOS (Universal Apple Silicon + Intel)** y **Linux (`.AppImage`, `.deb`, `.rpm`, `.tar.gz`)**.

---

## 🇪🇸 Novedades en v1.0.0
- **Aplicación de escritorio nativa (Rust + C `cubiomes`)** sin navegador ni Electron, rápida y 100% utilizable offline.
- **6 módulos completos**:
  - **Mapa de Semilla (`26.3`)**: Overworld, Nether y End, detección automática de mundos locales (`.minecraft/saves`, macOS `Application Support`, Linux Flatpak y PrismLauncher), Slime Chunks, Biomas Grandes, coordenadas enlazadas 8:1, Regla, Waypoints, pin con copia `/tp` y exportación a PNG. Escaneo priorizado para no perder nunca **Mansiones** (`dark_forest` y `pale_garden`), Strongholds ni estructuras raras.
  - **Equipamiento God y Yunque Vivo**: 29 presets con cálculo óptimo de árbol de fusión para evitar el tope de 39 niveles (*"Demasiado caro"*).
  - **Crafteos JEI/EMI y Alquimia**: 2,023 recetas navegables con árbol recursivo de materias primas y tabla visual completa de pociones (`Redstone`, `Glowstone`, `Pólvora`, `Aliento de Dragón`, `Ojo de Araña Fermentado` y flechas con efecto).
  - **Granjas Técnicas (113) + Creadores (31)**: checklist en stacks o unidades, detección local de `.litematic` y búsqueda adaptada a versión e idioma junto a creadores especializados con sus avatares de skin reales.
  - **Dupes & Glitches (34) + Pro Tips (17)**: estado por versión `26.3` y calculadora rápida de portales.
- **Soporte Multiplataforma completo**: rutas nativas de configuración y mundos en Windows, macOS y distros Linux (`Ubuntu/Debian`, `Fedora/RHEL/openSUSE`, `Arch`), tipografías del sistema para símbolos y CJK, y empaquetado automático vía GitHub Actions.
- **Detector de actualizaciones integrado**: comprueba silenciosamente GitHub Releases para avisarte cuando haya una nueva versión (`v1.x`, base lista para futuro auto-updater).

---

## 🇬🇧 What's New in v1.0.0
- **Cross-Platform Native Desktop Release**: builds for **Windows (x86_64)**, **macOS Universal (`arm64` + `x86_64` `.dmg`/`.zip`)**, and **Linux (`.AppImage`, `.deb`, `.rpm`, `.tar.gz`)**.
- **6 Full Modules**:
  - **Native Seed Map (`cubiomes` Java `26.3`)**: Overworld/Nether/End biome & terrain explorer, automatic `level.dat` world detection across Windows/macOS/Linux/Flatpak/PrismLauncher, rare-first structure scanning (including Mansions in `dark_forest` and `pale_garden`), 8:1 portal coordinate sync, ruler, custom waypoints, `/tp` quick-copy pin, and PNG export.
  - **God Gear & Living Anvil Optimizer**: 29 setups with mathematically optimal merge trees staying under Java's 39-level cap.
  - **JEI/EMI Recipes & Brewing Guide**: 2,023 recipes, recursive raw-material Crafting Tree, and complete potion modifier reference.
  - **113 Technical Farms + 31 Curated Creators**: stack/count checklists, local `.litematic` discovery, version-aware tutorial links, and specialty-tagged technical creators.
  - **34 Dupes/Glitches + 17 Pro Tips**: status badges for `26.3` and built-in Nether Portal Sync Calculator.
- **Built-in Release Checker**: unobtrusively checks GitHub Releases on launch so you stay notified of `v1.x` updates.

> **macOS first-launch tip**: Control-click (right-click) -> **Open** on the first launch (or run `xattr -cr "/Applications/Minecraft Companion.app"`), since this build is ad-hoc signed.
