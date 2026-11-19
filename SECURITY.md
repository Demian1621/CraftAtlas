# Security Policy / Política de Seguridad

## 🛡️ Garantías de diseño de CraftAtlas

- **Ejecución local y sin telemetría**: la aplicación funciona completamente en local; las únicas conexiones salientes son:
  1. Comprobación en segundo plano de nuevas versiones vía HTTPS (`TLS 1.2+`) hacia `https://api.github.com/repos/Demian1621/CraftAtlas/releases/latest`.
  2. Enlaces explícitos abiertos a pedido del usuario (`https://www.youtube.com/...`, `https://www.bilibili.com/...` o `https://github.com/Demian1621/CraftAtlas/releases/...`).
- **Validación estricta**:
  - Ninguna URL sin esquema `https://` válido puede abrirse en el navegador.
  - En Windows, las consultas y descargas de Releases invocan exclusivamente `%SystemRoot%\System32\curl.exe` para evitar secuestro de binario desde `PATH` o carpeta de descargas.
  - **Auto-updater *fail-closed* con SHA-256 nativo (FIPS 180-4)**: el actualizador in-app verifica cabeceras mágicas del ejecutable (`MZ` / `ELF`), compara el digest SHA-256 en tiempo constante contra el digest oficial de GitHub / `SHA256SUMS.txt` (tope 64 MiB) y descarta el archivo inmediatamente si falta el checksum o no coincide.
  - Lectura de `level.dat` (máx. 16 MiB comprimidos / 32 MiB descomprimidos) y de esquemáticos `.litematic` (máx. 32 MiB comprimidos / 64 MiB descomprimidos, máx. 64M bloques con aritmética `checked_mul`) acotada contra *decompression bombs* y *integer overflows*.
  - **Temas de video personalizados (`.cavf` / `ffmpeg`)**: tope estricto de `250 MiB` en el archivo origen, máximo `12 s` (`854×480` a `12 FPS`, máximo `360` frames / `24 MiB` cacheados), validación estricta de extensión (`.mp4`, `.webm`, `.mov`, `.mkv`, `.gif`, `.cavf`), invocación directa sin *shell* resolviendo `ffmpeg` exclusivamente desde rutas absolutas del sistema (`PATH`, nunca desde la carpeta actual), escritura atómica en `%APPDATA%\craft-atlas\video_themes\custom.cavf` y decodificación segura *fail-closed* de cabeceras y offsets.
  - Guardado atómico de preferencias (`prefs.json.tmp` → `prefs.json`, permisos `0600` en Unix).
- **Cadena de suministro (CI/CD)**:
  - Todos los workflows de GitHub Actions usan permisos mínimos (`contents: read`, elevando `contents: write`, `id-token: write` y `attestations: write` solo en el job de Release), Actions fijadas por commit SHA exacto, `cargo clippy --all-targets -- -D warnings` en los 3 sistemas operativos, verificación SHA-256 de `appimagetool 1.9.1` y atestaciones criptográficas **SLSA / Sigstore Build Provenance** (`actions/attest-build-provenance`).

---

## 🔍 Cómo verificar los archivos descargados (`SHA256SUMS.txt` y Build Provenance)

Cada Release incluye `SHA256SUMS.txt` y atestaciones criptográficas de GitHub. Podés comprobar la integridad exacta del binario antes de abrirlo:

- **Windows (PowerShell)**:
  ```powershell
  Get-FileHash .\craft-atlas-v1.2.0-windows-x86_64.exe -Algorithm SHA256
  ```
- **macOS**:
  ```bash
  shasum -a 256 craft-atlas-v1.2.0-macos-universal.dmg
  ```
- **Linux**:
  ```bash
  sha256sum -c SHA256SUMS.txt --ignore-missing
  ```
- **Verificar procedencia oficial de GitHub Actions (`gh` CLI)**:
  ```bash
  gh attestation verify craft-atlas-v1.2.0-windows-x86_64.exe --repo Demian1621/CraftAtlas
  ```

---

## 📩 Reportar una vulnerabilidad

Si detectás algún problema de seguridad o comportamiento anómalo, usá **Private Vulnerability Reporting** en GitHub Security (cuando el repositorio sea público) o abrí un reporte privado al propietario (`Demian1621`) sin publicar exploits activos antes de que exista un parche.
