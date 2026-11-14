# Minecraft Companion `v1.0.1` — Hardening de Seguridad & CI

Actualización de mantenimiento y seguridad sobre `v1.0.0` sin cambiar el flujo de uso:

## 🇪🇸 Qué mejora en `v1.0.1`
- **Hardening del comprobador de actualizaciones ([src/updater.rs](https://github.com/Demian1621/CraftAtlas/blob/main/src/updater.rs))**:
  - En Windows resuelve exclusivamente `%SystemRoot%\System32\curl.exe` (evita *binary planting* desde Descargas/`PATH`); en macOS/Linux usa rutas de sistema absolutas.
  - Exige `--proto =https --proto-redir =https --tlsv1.2` y tamaño máximo de respuesta (`256 KiB`).
  - Restringe las URLs de actualización estrictamente a `https://github.com/Demian1621/CraftAtlas/releases/...`, valida formato del tag, limpia caracteres de control/BiDi en notas de versión y parsea metadatos SHA-256 de assets para el futuro auto-updater.
- **Validación de enlaces externos y archivos locales**:
  - Todos los enlaces salientes validan `https://` limpio antes de abrirse en el navegador.
  - Protección contra *gzip bombs* al leer `level.dat` (límite 16 MiB comprimido / 32 MiB descomprimido) y validación estricta de hashes SHA-1 en `assets/indexes` locales.
  - Guardado atómico de `prefs.json` (`.tmp` → rename) con permisos `0600` en sistemas Unix.
- **Cadena de suministro en GitHub Actions**:
  - Todas las Actions fijadas por **commit SHA** exacto, `persist-credentials: false`, permisos mínimos (`contents: read` global, `contents: write` solo en el job de Release) y verificación SHA-256 fija para `appimagetool 1.9.1`.
  - Documentación añadida en [SECURITY.md](https://github.com/Demian1621/CraftAtlas/blob/main/SECURITY.md).

## 🇬🇧 What's improved in `v1.0.1`
- **Hardened Release Checker**: resolves `%SystemRoot%\System32\curl.exe` exclusively on Windows (no CWD/PATH binary planting), enforces HTTPS-only redirects + TLS 1.2+ + 256 KiB cap, scopes `html_url` strictly to `Demian1621/CraftAtlas/releases/`, strips control/BiDi characters, and extracts SHA-256 release asset digests ahead of future auto-update support.
- **Input & Local File Guards**: `https://` allowlist on external links, bounded gzip decompression on `level.dat` (prevents decompression bombs), strict 40-hex validation for Minecraft asset hashes, and atomic `prefs.json` saves (`0600` on Unix).
- **CI/CD Supply-Chain Hardening**: SHA-pinned GitHub Actions, least-privilege workflow `permissions`, `env:` input isolation, and pinned SHA-256 verification for `appimagetool 1.9.1`.
