# CraftAtlas — Release Notes (`v1.2.1`)

> **[Español](#novedades-en-español) · [English](#whats-new-in-english)**

---

## Novedades en Español

### 🎨 Galería Visual de Temas + Intensidad + Fondo Propio (100% Nativo)
- **Nueva Galería Visual de Temas:** reemplazamos el desplegable plano de texto por un botón compacto con muestras de color en la barra superior que abre una **grilla visual en vivo** con miniaturas `16:9` de los **10 temas integrados**, paleta de 3 acentos, insignia de efecto y vista previa instantánea.
- **Control de Intensidad del Fondo (`20% – 100%`):** ajustá en tiempo real cuánta luminosidad querés en el paisaje detrás de las tarjetas translúcidas para priorizar atmósfera o máximo contraste de lectura.
- **Mi Fondo Propio (`.png` / `.jpg` nativo, sin FFmpeg):** elegí cualquier captura de tu mundo o shaderpack (`≤ 16 MiB`, validación segura en Rust puro y guardado atómico local) con vista previa instantánea y botón rápido para activarlo o quitarlo.

### ✨ Adiós loops `.cavf` trabados → Fondos HD (`1376×768`) + Efectos Vivos a 30 FPS
- **Calidad HD nativa sin cortes ni cámara girando en círculos:** reemplazamos los clips `.cavf` repetitivos de baja resolución (`854×480 @ 12 FPS`) por las **4 pinturas HD (`1376×768`)** estables sobre GPU más ~2 MB menos en el ejecutable final:
  - **Ciudad Antigua (Deep Dark):** anillos expansivos tipo *Sonic Boom* desde el portal central, nodos sculk respirando y esporas luminosas ascendentes.
  - **Trial Chambers:** 4 rayos de luz oblicuos desde el tragaluz superior, parpadeo cálido en los *Trial Spawners* y chispas ascendentes de cobre/ámbar.
  - **Jardín Pálido:** 3 capas de neblina volumétrica en movimiento horizontal y ojos ámbar de **Creaking** parpadeando entre las sombras.
  - **Monumento Oceánico:** 6 haces de cáusticas submarinas, pulso en linternas de mar, burbujas reales con brillo y siluetas lejanas de Guardianes patrullando.
- Mejoras atmosféricas extra en *Aurora del End* (cortinas de aurora ondulantes), *Noche Estrellada* (estrella fugaz periódica) y *Cerezo Atardecer* (haces cálidos + pétalos).

### 🔧 Correcciones de UI y CI Multiplataforma
- **God Armor ([god_armor.rs](src/ui/tabs/god_armor.rs)):** evitamos el corte vertical del chip de encantamiento largo (*Reparación*) envolviendo cada etiqueta completa.
- **Cero warnings en GitHub Actions:** actualización a Actions nativas de **Node 24** (`checkout@v7`, `upload-artifact@v7`, `download-artifact@v7`) junto con el fix en `build.rs` para Ubuntu y macOS.

---

## What's New in English

### 🎨 Visual Theme Gallery Modal + Backdrop Brightness + Native Custom Wallpaper
- **New Visual Theme Gallery:** replaced the narrow text-only `ComboBox` with a color-swatch pill button in the top bar that opens a live **`16:9` thumbnail gallery** across all **10 built-in themes**, showing color palette swatches, live FX tags, and instant behind-modal preview.
- **Live Backdrop Intensity Slider (`20% – 100%`):** fine-tune scene brightness vs. UI panel contrast dynamically in real time.
- **Custom Image Wallpaper (`.png` / `.jpg`, zero external tools):** pick any screenshot from your world or shaderpack (`≤ 16 MiB`, fail-closed pure-Rust header validation & atomic local storage) with instant preview, quick toggle, and one-click removal.

### ✨ Replaced Stuttery `.cavf` Loops with HD (`1376×768`) Scenes & 30 FPS Living Biome FX
- Eliminated the repetitive 24-frame `.cavf` loops and circular camera wobble, upgrading **Deep Dark**, **Trial Chambers**, **Pale Garden**, and **Ocean Monument** to crisp **`1376×768`** scenes (~2 MB smaller release binaries) paired with bespoke **30 FPS GPU atmospheric FX**:
  - **Deep Dark:** expanding Sculk sonic pulse rings from the central portal + breathing cyan nodes & glowing spores.
  - **Trial Chambers:** angled skylight sunbeams, flickering Trial Spawner firelight & rising copper/amber embers.
  - **Pale Garden:** multi-speed drifting mist ribbons & blinking amber **Creaking** eyes emerging from tree shadows.
  - **Ocean Monument:** sweeping underwater caustic beams, Sea Lantern pulses, specular rising bubbles & distant patrolling Guardians.
- Upgraded scenic FX on *End Aurora* (flowing dual aurora ribbons), *Starry Ruin* (periodic shooting stars), and *Cherry Sunset* (soft sun rays).

### 🔧 UI & Cross-Platform CI Polish
- **God Armor chip wrapping fix:** prevents long enchantment badges like *Reparación* from wrapping mid-word.
- **Zero-warning CI/Release pipelines:** upgraded pinned GitHub Actions to **Node 24** (`actions/checkout` v7.0.1, `upload-artifact` v7.0.2, `download-artifact` v7.0.0) alongside Unix `build.rs` cfg-gating.
