<div align="center">

# CraftAtlas (`v1.1.0`)

**La navaja suiza de escritorio para sobrevivir, optimizar y romper límites en Minecraft Java (`26.3`) — nativa en Rust + C, 100% offline y multiplataforma.**

[![CI](https://github.com/Demian1621/CraftAtlas/actions/workflows/ci.yml/badge.svg)](https://github.com/Demian1621/CraftAtlas/actions/workflows/ci.yml)
[![Release](https://github.com/Demian1621/CraftAtlas/actions/workflows/release.yml/badge.svg)](https://github.com/Demian1621/CraftAtlas/actions/workflows/release.yml)

[Español](#-por-qué-existe-craftatlas) · [English Version](README.en.md) · [Seguridad](SECURITY.md) · [Releases](https://github.com/Demian1621/CraftAtlas/releases) · [Feedback](https://github.com/Demian1621/CraftAtlas/issues/new/choose) · [Apoyar en Ko-fi](https://ko-fi.com/titusdb)

</div>

---

## 🎯 ¿Por qué existe CraftAtlas?

Cualquiera que juegue Minecraft técnico o survival avanzado conoce la rutina de tener **14 pestañas abiertas en el navegador** mientras juega: Chunkbase para buscar una Mansión o un Stronghold, la wiki para recordar el orden de encantamientos sin que el yunque te tire *"Demasiado caro"*, JEI o una tabla de alquimia en otra ventana, calculadora de portales 8:1 del Nether en otra, y un bloc de notas con los materiales de la próxima granja.

**CraftAtlas** *(antes llamado Minecraft Companion en `v1.0.0`–`v1.0.2`)* junta todo eso en **una sola app nativa, ligera y sin navegador de por medio**, compilada directamente en **Rust (`eframe`/`egui`)** junto al motor nativo **C (`cubiomes`)**. Abre instantáneamente, no consume RAM como Electron, detecta tus mundos locales de `.minecraft/saves` y te acompaña sin anuncios ni pantallas de carga eternas.

---

## 🧭 Qué incluye la v1.1.0

### 1. 🗺️ Mapa de Semilla Nativo (`cubiomes` C Engine)
- **Overworld, Nether y End** con renderizado nativo de biomas, terreno 3D topográfico y **Slime Chunks**.
- **Lectura directa de tus mundos locales (`level.dat`)**: detecta automáticamente carpetas de Minecraft oficial, macOS `Application Support`, Flatpak y **PrismLauncher** (ordenados por último juego) para cargar tu semilla al instante sin escribirla a mano.
- **Semillas guardadas e historial (pensado para Servers):** guardá tus semillas con nombre (por ejemplo servidores SMP que no existen en `.minecraft/saves`) para abrirlas en 1 clic y consultá el historial automático de las últimas 20 semillas exploradas.
- **Última posición guardada y skin personalizada (`.png`)**: muestra en el mapa la última posición guardada del jugador (`Player.Pos` + dimensión) aclarando explícitamente que es la última posición guardada (no en vivo), con botón de recarga y carga de tu archivo de skin `.png` (`64x64`, `64x32` o HD) para mostrar tu cara real en el mapa.
- **Escaneo inteligente por prioridad de estructuras**: Strongholds, **Mansiones del Bosque** (incluyendo biomas `dark_forest` y `pale_garden` de Java 26.3), Monumentos Oceánicos, **Ciudades Antiguas (ícono de cabeza de Warden)**, Trial Chambers, Bastiones, Fortalezas y End Cities con Barco.
- **Sincronización Overworld ↔ Nether (8:1)**, herramienta **Regla** para medir bloques exactos, **marcadores (waypoints) personalizados** (con ícono de cama roja clásica para `Casa / Cama`), pin temporal con copia instantánea de comando `/tp` (click derecho en cualquier coordenada) y **exportación a PNG**.

### 2. ⚒️ Equipamiento God, Yunque Vivo y Libros Encantados (29 presets · 43 libros)
- Configuraciones completas de armadura, herramientas, armas (incluyendo **Lanza / Estocada**) y élitros con qué hace y para qué sirve cada encantamiento y todas sus incompatibilidades.
- **Nueva guía de Libros Encantados conmutada**: los **43 encantamientos oficiales de Minecraft Java (`26.3`)**, sus fuentes (Mesa de Encantamientos, Aldeano Librero, Pesca, Trueque Piglin y Cofres/Bóvedas), incompatibilidades, filtro específico y alerta clara para los **libros que NO venden los aldeanos** (*Sigilo rápido / Swift Sneak*, *Velocidad de alma / Soul Speed*, *Aeroimpulso / Wind Burst*).
- **Optimizador matemático real de Yunque Java**: calcula el **árbol de fusión óptimo** frente al orden ingenuo paso a paso para ahorrar niveles de XP y **no superar nunca el tope de 39 niveles ("Demasiado caro")**.
- Incluye simulación interactiva **Yunque Vivo** paso a paso para forjar tu pieza sin equivocarte.

### 3. 📖 Crafteos JEI / EMI + Alquimia Completa (2,023 recetas)
- Buscador instantáneo estilo **JEI / EMI** (`Crafteos` y `Usos`) con historial de navegación atrás/adelante, soporte para tags alternativos y **Árbol de Crafteo recursivo** con desglose exacto de materias primas.
- Cubre mesa de crafteo, horno, alto horno, ahumador, herrería, cortapiedras y **Mesa de Alquimia** con una guía visual completa de pociones: base rara (verruga del Nether), Redstone (`+Tiempo`), Glowstone (`+Nivel II`), Pólvora (`Lanzadora`), Aliento de Dragón (`Persistente`), Ojo de Araña Fermentado (`Corrupción`) y Flechas con Efecto.

### 4. 🌾 Granjas Técnicas y Creadores Especializados (113 granjas · 31 creadores)
- Filtrables por etapa (`Early`, `Mid`, `End Game`) y categoría, con lista de materiales conmutable entre **Stacks (x64)** y **conteo exacto** y checklist persistente.
- Detecta tus archivos locales de **Litematica (`.litematic` / `.schematic`)**.
- Botón inteligente de búsqueda en YouTube que adapta automáticamente idioma y versión activa de Minecraft (`26.3`), acompañado de un directorio curado de **31 creadores técnicos** de habla hispana, inglesa y china con sus caras reales de skin y etiquetas de especialidad (`Granjas`, `Dupes/Exploits`, `Redstone`).

### 5. ⚡ Dupes, Glitches y Calculadora de Portales (34 mecánicas)
- Fichas técnicas diferenciando lo que **funciona en 26.3** frente a mecánicas históricas o parcheadas, con componentes necesarios, explicación técnica y **calculadora instantánea de enlace de portales Overworld ↔ Nether**.

### 6. 💡 Pro Tips y Experiencia Personalizable (17 guías · 6 temas · 3 idiomas)
- **2,308 íconos** empaquetados en atlas de alta nitidez (sin borrosidad), audio sintetizado desactivable, **6 fondos panorámicos/escénicos animados** (*Cueva Profunda, Panorama Salvaje, Nether Carmesí, Aurora del End, Cerezo Atardecer, Noche Estrellada*), buscador universal **`Ctrl + F`**, barra de **Favoritos**, idiomas **`ES-MX`**, **`ES-ES`** y **`EN-US`**, y **comprobador integrado de nuevas versiones** desde GitHub Releases (preparado para evolucionar a auto-actualizador en futuras versiones).

---

## 💻 Descarga e Instalación (Windows · macOS · Linux)

Descarga el instalador o ejecutable de tu sistema desde la sección de **[Releases](https://github.com/Demian1621/CraftAtlas/releases)**:

### 🪟 Windows (x86_64)
- **`.exe` directo**: doble clic y listo (portable, no requiere instalación).
- **`.zip`**: descomprime y ejecuta `craft-atlas.exe`.

### 🍎 macOS (Universal: Apple Silicon M1/M2/M3/M4 + Intel)
1. Descarga **`craft-atlas-v1.1.0-macos-universal.dmg`** (o el `.zip`).
2. Arrastra **CraftAtlas.app** a tu carpeta **Aplicaciones**.
3. **Nota de primera apertura (Gatekeeper)**: como esta release aún no cuenta con firma paga de Apple Developer, haz **clic derecho (o Ctrl + clic) → Abrir → Abrir**, o ejecuta una sola vez en Terminal:
   ```bash
   xattr -cr "/Applications/CraftAtlas.app"
   ```

### 🐧 Linux (Todas las distros)
| Formato | Distros recomendadas | Cómo usar |
| --- | --- | --- |
| **`.AppImage`** *(Universal)* | Ubuntu, Fedora, Arch, Mint, Pop!_OS, openSUSE, SteamOS | `chmod +x craft-atlas-v1.1.0-linux-x86_64.AppImage && ./craft-atlas-v1.1.0-linux-x86_64.AppImage` |
| **`.deb`** | Ubuntu, Debian, Linux Mint, Pop!_OS, Zorin | `sudo apt install ./craft-atlas_1.1.0_amd64.deb` |
| **`.rpm`** | Fedora, RHEL, Rocky, openSUSE | `sudo dnf install ./craft-atlas-1.1.0-1.x86_64.rpm` |
| **`.tar.gz`** *(Portable)* | Arch Linux, Manjaro, NixOS, genérico | Extrae y ejecuta `./craft-atlas` o `./install-user.sh` |

> **Nota Linux y verificación SHA-256**: en Linux requiere librerías gráficas estándar (`OpenGL` / `Wayland` o `X11`) y audio `ALSA`. Podés verificar la integridad de cualquier descarga comparando su hash con `SHA256SUMS.txt` (ver [SECURITY.md](SECURITY.md)).

---

## 🛠️ Compilar desde el código fuente

Requiere **Rust (`cargo` / `rustc` estables)** y compilador C (`gcc` / `clang` / MSVC) para compilar `cubiomes`:

```bash
# En Linux (Ubuntu/Debian) instala primero dependencias de sistema:
sudo apt-get install -y build-essential pkg-config libasound2-dev \
  libxkbcommon-dev libwayland-dev libgl1-mesa-dev libxcursor-dev \
  libxrandr-dev libxi-dev libx11-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev

# Compilar y ejecutar pruebas:
cargo test --locked
cargo build --release --locked
```

---

## ⚡ Novedades de `v1.1.0`

- **Semillas guardadas e historial para Servidores + Última posición y Skin `.png` propia:** Guardá seeds con nombre propio (ideal para servidores online sin `level.dat` local), consultá el historial de tus últimas 20 semillas, visualizá tu última posición guardada (`Player.Pos`, aclarando que no es en vivo) y cargá o arrastrá tu skin `.png`.
- **Enciclopedia completa de Libros Encantados (43 encantamientos Java `26.3`):** Sub-vista dedicada dentro de **Equipamiento God y Yunque** con ícono de libro encantado + íconos de los ítems compatibles, fuentes de obtención, incompatibilidades, mejor método y filtro para los libros que no dan los aldeanos (*Sigilo rápido*, *Velocidad de alma*, *Aeroimpulso*).
- **Íconos y distribución renovada en el Mapa de Semilla:** Panel lateral limpio en 3 pestañas (**Marcadores** primero arriba de todo, **Capas** en 2 columnas y **Mundos / Skin**), selector desplegable unificado, nuevo ícono de **cama roja clásica** para `Casa / Cama` e ícono de **cabeza de Warden** para la **Ciudad Antigua**.
- Además mantiene todas las optimizaciones de `v1.0.2` (**~0% de CPU en segundo plano**, **30 FPS activos**, botones de **Feedback/Ko-fi** y balance del yunque). Ver notas completas en [RELEASE_v1.1.0.md](docs/RELEASE_v1.1.0.md).

---

## 💬 Feedback y apoyo al proyecto

CraftAtlas es **100% gratis, sin anuncios y sin funciones bloqueadas detrás de pagos**. Todo está disponible desde el primer día.

- **¿Encontraste un bug o querés proponer una idea?** Abrí un reporte o sugerencia desde [GitHub Issues](https://github.com/Demian1621/CraftAtlas/issues/new/choose) o sumate a las [GitHub Discussions](https://github.com/Demian1621/CraftAtlas/discussions).
- **¿Te ahorró tiempo y querés invitar un café? (Opcional):** Podés apoyar el proyecto en [Ko-fi (`https://ko-fi.com/titusdb`)](https://ko-fi.com/titusdb) o usar el botón **Apoyar** en la barra superior de la app.

---

## 🚀 Actualizaciones futuras (Roadmap v1.x)

La arquitectura `v1.x` ya incluye verificación asíncrona endurecida por TLS 1.2+ contra **GitHub Releases** (`v1.1.0` en la barra superior). En próximas versiones (`v1.2+`) evolucionará a **descarga e instalación automática in-app con verificación SHA-256**, además de ampliar versiones históricas del registro de Minecraft.

---

<div align="center">
Hecho con Rust, C y pasión por el survival técnico · <b>Demian1621</b>
</div>
