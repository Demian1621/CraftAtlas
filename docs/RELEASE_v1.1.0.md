# CraftAtlas — Release Notes (`v1.1.0`)

> **[Español](#novedades-en-español) · [English](#whats-new-in-english)**

---

## Novedades en Español

### 🏷️ Nuevo nombre oficial: CraftAtlas (`craft-atlas`)
- **Rebranding de proyecto:** La aplicación pasa de llamarse *Minecraft Companion* a **`CraftAtlas`** (slug de paquete y binarios: `craft-atlas`).
- **Migración automática de configuración y skin:** Si ya usabas versiones anteriores (`v1.0.0` – `v1.0.2`), `CraftAtlas` lee automáticamente tus preferencias, favoritos y skin guardada desde la carpeta anterior si aún no existe configuración nueva en `%APPDATA%\craft-atlas` (Windows) o `~/.config/craft-atlas` (Linux/macOS).

### 🗺️ Mapa de Semilla: Servidores Guardados, Historial, Posición Guardada y Skin Propia
- **Semillas guardadas e historial (ideal para Servers):** Como los servidores online no tienen `level.dat` en tu `.minecraft/saves`, ahora podés **guardar semillas con nombre propio** (marcando si son `Server Online` o mundo local, conservando `Biomas Grandes`) para cargarlas con 1 clic, y consultar un **historial automático de las últimas 20 semillas exploradas** (manuales, aleatorias o locales).
- **Tu última posición en el mundo (`level.dat`):** Al seleccionar o recargar un mundo local de `.minecraft/saves`, el mapa ahora lee `Player.Pos` y `Player.Dimension`, coloca tu marcador de jugador y te permite centrar el mapa o copiar `/tp` al instante.
- **Aviso explícito de sincronización:** La app aclara que se muestra la **última posición guardada en disco (no en vivo)**, junto con botón rápido de recarga de `.minecraft`.
- **Sube tu skin `.png` como en Minecraft:** Desde la barra lateral del Mapa de Semilla o arrastrando el archivo `.png` a la ventana (`64x64`, `64x32` o HD múltiplo), recorta automáticamente tu cara 8x8 + capa de pelo/casco y la guarda localmente (con respaldo de Steve por defecto y botón para restablecer).
- **Íconos y distribución mejorada en el Mapa de Semilla:** Panel lateral organizado en 3 pestañas (**Marcadores** primero arriba de todo con navegación rápida, **Capas** en grilla limpia de 2 columnas con presets rápidos y **Mundos y Skin**) junto a un selector desplegable unificado en la barra superior. El marcador **Casa / Cama** ahora usa la **cama roja clásica isométrica** y **Ciudad Antigua** se identifica con una nueva **cabeza de Warden** en pixel art.

### 📚 Nueva Guía de Libros Encantados (Equipamiento God y Yunque)
- **Vista conmutada dentro de "Equipamiento God y Yunque":** Alterná fácilmente entre el Optimizador de Yunque y la **enciclopedia completa de 43 libros encantados** de Minecraft Java (`26.3`, incluyendo **Estocada / Lunge** para lanza).
- **Ícono de libro encantado + ítems compatibles:** Cada entrada muestra el ícono del **libro encantado** acompañado de los **íconos reales de todos los ítems** donde puede aplicarse (casco, pechera, espada, lanza, maza, arco, tridente, élitros, herramientas, etc.).
- **Cómo conseguirlos y cuál es el mejor método:** Cada libro detalla disponibilidad en **Mesa de Encantamientos, Aldeano Librero, Pesca, Trueque con Piglins y Cofres / Bóvedas**, además de sus incompatibilidades y la estrategia recomendada.
- **Filtro y alerta "Sin aldeano":** Resalta de forma clara los encantamientos que **no venden los aldeanos** (*Sigilo rápido / Swift Sneak*, *Velocidad de alma / Soul Speed* y *Aeroimpulso / Wind Burst*).
- **Integración con buscador global (`Ctrl + F`):** Podés buscar cualquier libro encantado directo desde la paleta rápida.

---

## What's New in English

### 🏷️ Official Rebrand: CraftAtlas (`craft-atlas`)
- **Project Rename:** Formerly published as *Minecraft Companion* (`v1.0.0` – `v1.0.2`), the app and repository are now officially named **`CraftAtlas`** (`Demian1621/CraftAtlas`).
- **Automatic Settings & Skin Migration:** Existing users upgrading from `v1.0.x` keep their saved pins, seed history, preferences, and custom skin via automatic fallback migration from the legacy config directory.

### 🗺️ Seed Map: Saved Server Seeds, History, Saved Position & Custom Skin
- **Saved Seeds & History (Multiplayer Ready):** Since online multiplayer servers don't write a local `level.dat`, you can now **bookmark named seeds** (marking them as `Online Server` or local worlds, preserving `Large Biomes`) for one-click access and browse an **automatic history of your last 20 visited seeds**.
- **Last Saved `level.dat` Location:** Selecting or reloading a local `.minecraft/saves` world reads `Player.Pos` and `Player.Dimension`, marks your player spot on the Seed Map, and lets you jump to it or copy `/tp`.
- **Explicit Non-Live Label:** Clearly states that the marker reflects the **last saved disk position (not live)** alongside a manual `.minecraft` reload button.
- **Upload Your `.png` Skin:** Pick via native file dialog or drag-and-drop any `64x64`, `64x32`, or HD Minecraft skin `.png`; extracts the `8x8` face + hat overlay and persists it locally (with instant reset to classic Steve).
- **Cleaner Seed Map Layout & Upgraded Icons:** Reorganized sidebar into 3 focused tabs (**Waypoints** pinned first at top with quick coordinate jump, **Layers** in a clean 2-column tile grid with instant presets, and **Worlds & Skin**) paired with a unified world/server dropdown in the top bar. Replaced the `Home / Bed` red wool placeholder with a **classic isometric red bed**, and replaced `Ancient City`'s shrieker top with a crisp **Warden head** icon.

### 📚 New Enchanted Books Encyclopedia (God Gear & Anvil Tab)
- **Switched Sub-View:** Toggle seamlessly between **God Gear & Anvil** and all **43 official Minecraft Java (`26.3`) enchanted books** (including **Lunge** for spears).
- **Enchanted Book Icon + Compatible Gear Icons:** Every enchantment shows the **Enchanted Book** sprite paired with **item icons** showing every compatible piece of gear (helmets, chestplates, swords, spears, maces, bows, tridents, elytra, tools, etc.).
- **Complete Acquisition & Strategy Guide:** Shows Enchanting Table, Librarian Villager, Fishing, Piglin Bartering, and Chest/Vault loot sources, mutual incompatibilities, and the optimal farming method per book.
- **"No Villager" Filter & Warning:** Highlights books that Librarian Villagers never trade (*Swift Sneak*, *Soul Speed*, *Wind Burst*).
- **Global Search Integration (`Ctrl + F`):** Search enchanted books straight from the universal command palette.
