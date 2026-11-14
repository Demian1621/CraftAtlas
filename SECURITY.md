# Security Policy / Política de Seguridad

## 🛡️ Garantías de diseño de CraftAtlas

- **Ejecución local y sin telemetría**: la aplicación funciona completamente en local; las únicas conexiones salientes son:
  1. Comprobación en segundo plano de nuevas versiones vía HTTPS (`TLS 1.2+`) hacia `https://api.github.com/repos/Demian1621/CraftAtlas/releases/latest`.
  2. Enlaces explícitos abiertos a pedido del usuario (`https://www.youtube.com/...`, `https://www.bilibili.com/...` o `https://github.com/Demian1621/CraftAtlas/releases/...`).
- **Validación estricta**:
  - Ninguna URL sin esquema `https://` válido puede abrirse en el navegador.
  - En Windows, la consulta de Releases invoca exclusivamente `%SystemRoot%\System32\curl.exe` para evitar secuestro de binario desde `PATH` o carpeta de descargas.
  - Lectura de `level.dat` acotada contra *decompression bombs* (máx. 16 MiB comprimidos / 32 MiB descomprimidos).
  - Guardado atómico de preferencias (`prefs.json.tmp` → `prefs.json`, permisos `0600` en Unix).
- **Cadena de suministro (CI/CD)**:
  - Todos los workflows de GitHub Actions usan permisos mínimos (`contents: read`, elevando `contents: write` solo en la publicación de la Release), Actions fijadas por commit SHA exacto y verificación SHA-256 de herramientas externas (`appimagetool 1.9.1`).

---

## 🔍 Cómo verificar los archivos descargados (`SHA256SUMS.txt`)

Cada Release incluye `SHA256SUMS.txt`. Podés comprobar la integridad exacta del instalador antes de abrirlo:

- **Windows (PowerShell)**:
  ```powershell
  Get-FileHash .\craft-atlas-v1.1.0-windows-x86_64.exe -Algorithm SHA256
  ```
- **macOS**:
  ```bash
  shasum -a 256 craft-atlas-v1.1.0-macos-universal.dmg
  ```
- **Linux**:
  ```bash
  sha256sum -c SHA256SUMS.txt --ignore-missing
  ```

---

## 📩 Reportar una vulnerabilidad

Si detectás algún problema de seguridad o comportamiento anómalo, usá **Private Vulnerability Reporting** en GitHub Security (cuando el repositorio sea público) o abrí un reporte privado al propietario (`Demian1621`) sin publicar exploits activos antes de que exista un parche.
