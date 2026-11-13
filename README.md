<div align="center">

# Minecraft Companion (`v1.0.2`)

**La navaja suiza de escritorio para sobrevivir, optimizar y romper límites en Minecraft Java (`26.3`) — nativa en Rust + C, 100% offline y multiplataforma.**

[![CI](https://github.com/Demian1621/minecraft-companion/actions/workflows/ci.yml/badge.svg)](https://github.com/Demian1621/minecraft-companion/actions/workflows/ci.yml)
[![Release](https://github.com/Demian1621/minecraft-companion/actions/workflows/release.yml/badge.svg)](https://github.com/Demian1621/minecraft-companion/actions/workflows/release.yml)

[Español](#-por-qué-existe-minecraft-companion) · [English Version](README.en.md) · [Seguridad](SECURITY.md) · [Releases](https://github.com/Demian1621/minecraft-companion/releases) · [Feedback](https://github.com/Demian1621/minecraft-companion/issues/new/choose) · [Apoyar en Ko-fi](https://ko-fi.com/titusdb)

</div>

---

## 🎯 ¿Por qué existe Minecraft Companion?

Cualquiera que juegue Minecraft técnico o survival avanzado conoce la rutina de tener **14 pestañas abiertas en el navegador** mientras juega: Chunkbase para buscar una Mansión o un Stronghold, la wiki para recordar el orden de encantamientos sin que el yunque te tire *"Demasiado caro"*, JEI o una tabla de alquimia en otra ventana, calculadora de portales 8:1 del Nether en otra, y un bloc de notas con los materiales de la próxima granja.

**Minecraft Companion** junta todo eso en **una sola app nativa, ligera y sin navegador de por medio**, compilada directamente en **Rust (`eframe`/`egui`)** junto al motor nativo **C (`cubiomes`)**. Abre instantáneamente, no consume RAM como Electron, detecta tus mundos locales de `.minecraft/saves` y te acompaña sin anuncios ni pantallas de carga eternas.

---

## 🧭 Qué incluye la v1.0.2

### 1. 🗺️ Mapa de Semilla Nativo (`cubiomes` C Engine)
- **Overworld, Nether y End** con renderizado nativo de biomas, terreno 3D topográfico y **Slime Chunks**.
- **Lectura directa de tus mundos locales (`level.dat`)**: detecta automáticamente carpetas de Minecraft oficial, macOS `Application Support`, Flatpak y **PrismLauncher** para cargar tu semilla al instante sin escribirla a mano.
- **Escaneo inteligente por prioridad de estructuras**: Strongholds, **Mansiones del Bosque** (incluyendo biomas `dark_forest` y `pale_garden` de Java 26.3), Monumentos Oceánicos, Ciudades Antiguas, Trial Chambers, Bastiones, Fortalezas y End Cities con Barco.
- **Sincronización Overworld ↔ Nether (8:1)**, herramienta **Regla** para medir bloques exactos, **marcadores (waypoints) personalizados**, pin temporal con copia instantánea de comando `/tp` (click derecho en cualquier coordenada) y **exportación a PNG**.

### 2. ⚒️ Equipamiento God y Yunque Vivo (29 presets)
- Configuraciones completas de armadura, herramientas, armas y élitros con qué hace y para qué sirve cada encantamiento y todas sus incompatibilidades.
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
- **2,307 íconos** empaquetados en atlas de alta nitidez (sin borrosidad), audio sintetizado desactivable, **6 fondos panorámicos/escénicos animados** (*Cueva Profunda, Panorama Salvaje, Nether Carmesí, Aurora del End, Cerezo Atardecer, Noche Estrellada*), buscador universal **`Ctrl + F`**, barra de **Favoritos**, idiomas **`ES-MX`**, **`ES-ES`** y **`EN-US`**, y **comprobador integrado de nuevas versiones** desde GitHub Releases (preparado para evolucionar a auto-actualizador en futuras versiones).

---

## 💻 Descarga e Instalación (Windows · macOS · Linux)

Descarga el instalador o ejecutable de tu sistema desde la sección de **[Releases](https://github.com/Demian1621/minecraft-companion/releases)**:

### 🪟 Windows (x86_64)
- **`.exe` directo**: doble clic y listo (portable, no requiere instalación).
- **`.zip`**: descomprime y ejecuta `minecraft-companion.exe`.

### 🍎 macOS (Universal: Apple Silicon M1/M2/M3/M4 + Intel)
1. Descarga **`minecraft-companion-v1.0.2-macos-universal.dmg`** (o el `.zip`).
2. Arrastra **Minecraft Companion.app** a tu carpeta **Aplicaciones**.
3. **Nota de primera apertura (Gatekeeper)**: como esta release aún no cuenta con firma paga de Apple Developer, si macOS muestra aviso de desarrollador no identificado, haz **clic derecho (o Ctrl + clic) → Abrir → Abrir**, o ejecuta una sola vez en Terminal:
   ```bash
   xattr -cr "/Applications/Minecraft Companion.app"
   ```

### 🐧 Linux (Todas las distros)
| Formato | Distros recomendadas | Cómo usar |
| --- | --- | --- |
| **`.AppImage`** *(Universal)* | Ubuntu, Fedora, Arch, Mint, Pop!_OS, openSUSE, SteamOS | `chmod +x minecraft-companion-v1.0.2-linux-x86_64.AppImage && ./minecraft-companion-v1.0.2-linux-x86_64.AppImage` |
| **`.deb`** | Ubuntu, Debian, Linux Mint, Pop!_OS, Zorin | `sudo apt install ./minecraft-companion_1.0.2_amd64.deb` |
| **`.rpm`** | Fedora, RHEL, Rocky, openSUSE | `sudo dnf install ./minecraft-companion-1.0.2-1.x86_64.rpm` |
| **`.tar.gz`** *(Portable)* | Arch Linux, Manjaro, NixOS, genérico | Extrae y ejecuta `./minecraft-companion` o `./install-user.sh` |

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

## ⚡ Novedades de `v1.0.2`

- **~0% de CPU en segundo plano mientras jugás:** Al cambiar de ventana (Alt+Tab a Minecraft) o minimizar la app, se pausan automáticamente el panorama 360°, partículas y destellos animados; en uso activo corre a ~30 FPS (`33 ms`) para cuidar batería y temperatura en laptops.
- **Botones de Feedback y Apoyo en la barra superior:** Acceso directo a reportar errores o sugerir mejoras en GitHub, y enlace opcional a Ko-fi.

---

## 💬 Feedback y apoyo al proyecto

Minecraft Companion es **100% gratis, sin anuncios y sin funciones bloqueadas detrás de pagos**. Todo está disponible desde el primer día.

- **¿Encontraste un bug o querés proponer una idea?** Abrí un reporte o sugerencia desde [GitHub Issues](https://github.com/Demian1621/minecraft-companion/issues/new/choose) o sumate a las [GitHub Discussions](https://github.com/Demian1621/minecraft-companion/discussions).
- **¿Te ahorró tiempo y querés invitar un café? (Opcional):** Podés apoyar el proyecto en [Ko-fi (`https://ko-fi.com/titusdb`)](https://ko-fi.com/titusdb) o usar el botón **Apoyar** en la barra superior de la app.

---

## 🚀 Actualizaciones futuras (Roadmap v1.x)

La arquitectura `v1.x` ya incluye verificación asíncrona endurecida por TLS 1.2+ contra **GitHub Releases** (`v1.0.2` en la barra superior). En próximas versiones (`v1.1.0+`) evolucionará a **descarga e instalación automática in-app con verificación SHA-256**, además de ampliar versiones históricas del registro de Minecraft.

---

<div align="center">
Hecho con Rust, C y pasión por el survival técnico · <b>Demian1621</b>
</div>
