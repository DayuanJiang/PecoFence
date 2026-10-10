https://github.com/user-attachments/assets/c827f059-cfd7-4f6a-bed3-8b00411a7220

<p align="center">
  <strong>Una alternativa gratuita y de código abierto a Stardock Fences para Windows 11.</strong><br>
  Agrupa archivos en paneles de cristal, cambia de proyecto con pestañas y deja que tu agente de IA configure la distribución, la apariencia y las reglas de organización con la CLI integrada.
</p>

<p align="center">
  <a href="https://pecofence.jiang.jp/es/"><strong>Sitio web</strong></a>
  &nbsp;·&nbsp; <a href="#descarga-pecofence"><strong>Descarga PecoFence →</strong></a>
  &nbsp;·&nbsp; <a href="#tu-escritorio-configurado-por-tu-ia"><strong>IA + CLI</strong></a>
  &nbsp;·&nbsp; <a href="#míralo-en-acción">Míralo en acción</a>
  &nbsp;·&nbsp; <a href="../README.md">Documentación</a>
</p>

<p align="center">
  <a href="../../README.md">English</a>
  &nbsp;·&nbsp; <a href="README.zh-CN.md">简体中文</a>
  &nbsp;·&nbsp; <a href="README.zh-TW.md">繁體中文</a>
  &nbsp;·&nbsp; <a href="README.ja.md">日本語</a>
  &nbsp;·&nbsp; <a href="README.ko.md">한국어</a>
  &nbsp;·&nbsp; <a href="README.de.md">Deutsch</a>
  &nbsp;·&nbsp; <a href="README.fr.md">Français</a>
  &nbsp;·&nbsp; <strong>Español</strong>
  &nbsp;·&nbsp; <a href="README.pt-BR.md">Português (Brasil)</a>
  &nbsp;·&nbsp; <a href="README.ru.md">Русский</a>
</p>

---

<a id="pídele-a-tu-ia-que-lo-organice"></a>

## Tu escritorio, configurado por tu IA

**CLI incluida. Lista para tu agente de IA.**

Dile a tu agente de IA cómo quieres usar tu escritorio. Con `pecofence-cli`, incluida en la aplicación, Claude Code, Codex y Cursor pueden leer tu configuración actual y aplicar cambios directamente en PecoFence.

- **Configura con tus propias palabras.** Cambia temas, transparencia, tamaño de los iconos y ajustes globales, o modifica todos los grupos a la vez.
- **Organiza una vez y mantén el orden.** Crea grupos por proyecto, distribuye los iconos y agrega reglas que clasifiquen los archivos nuevos automáticamente.
- **Guarda tu configuración favorita.** Usa instantáneas para la distribución de los grupos y la exportación e importación de configuración para ajustes, reglas y distribuciones.

**Pruébalo con tu agente de IA**

Abre PecoFence y pega esta petición en tu agente de programación con IA:

> Configura mi escritorio con pecofence-cli. Primero lee la salida de pecofence-cli skill y pecofence-cli describe; luego revisa mis ajustes y grupos actuales. Haz una copia de mi configuración antes de cambiarla. Activa el modo oscuro y haz todos los grupos más transparentes.

La CLI está incluida. La edición de Microsoft Store agrega `pecofence-cli` al PATH. Con el ZIP portátil, indica a tu agente la ruta completa de `pecofence-cli.exe`.

<details>
<summary><strong>Conversación de ejemplo con un agente de programación</strong></summary>

> Pon los PDF de mi escritorio en un grupo Docs y clasifica allí también los PDF nuevos. Activa el modo oscuro y haz los grupos más transparentes.

```powershell
pecofence-cli config export "$env:USERPROFILE\pecofence-before-ai.json"
pecofence-cli snapshot save before-cleanup
pecofence-cli fence create --title Docs
pecofence-cli rule add --name PDFs --ext pdf --to Docs --index 0
pecofence-cli rule apply
pecofence-cli settings set theme dark
pecofence-cli fence set --all opacity clear
```

</details>

Para agentes y scripts: `describe` ofrece el catálogo de comandos y los esquemas JSON; `skill`, la guía del agente. Los resultados JSON indican qué cambió y los errores estructurados de la aplicación ayudan al agente a elegir el siguiente paso.

[Primeros pasos con la CLI →](../CLI.md#start-with-your-ai-agent)

## Dale un lugar a cada cosa

<p align="center">
  <img src="../assets/hero-es.png" alt="PecoFence — Carpetas de proyecto, un PDF real y diseños originales en grupos nativos de Liquid Glass." width="1280">
</p>

## Míralo en acción

### Una ventana. Varios espacios de trabajo.

Mantén juntos los grupos relacionados como pestañas. Pasa de Project a Ideas con un clic y,
cuando necesites más espacio, arrastra una pestaña fuera para convertirla en su propio grupo.

![Dos grupos se unen en pestañas, se cambia entre Project e Ideas y luego una pestaña se separa en un grupo independiente.](../assets/tabs.gif)

### Tu escritorio, a un atajo de distancia.

Presiona **Ctrl + Alt + Espacio** y la vista rápida trae tus grupos por encima de la aplicación actual.
Toma lo que necesites y presiona **Esc** para volver.

![La vista rápida muestra los grupos del escritorio sobre una aplicación; un clic fuera o abrir un archivo devuelve a la aplicación.](../assets/peek.gif)

<sub>Animaciones del <a href="https://pecofence.jiang.jp/es/manual/">manual</a>. Los GIF se repiten automáticamente.</sub>

## Pequeños detalles que mejoran el día a día

| Experiencia | Qué obtienes |
| :--- | :--- |
| **IA + CLI** | `pecofence-cli` — **Configura con tus propias palabras.** Cambia temas, transparencia, tamaño de los iconos y ajustes globales, o modifica todos los grupos a la vez. |
| **Menos tiempo ordenando** | Reglas por tipo de archivo, extensión, nombre, comodines, destino del acceso directo, hora y tamaño. Los archivos nuevos encuentran su grupo solos. |
| **Cristal a la medida de tu escritorio** | Temas Fluent y Liquid Glass, modos claro y oscuro, colores por grupo, opacidad y tinte de iconos. |
| **Archivos como siempre** | Menús contextuales del Explorador de archivos, arrastrar y soltar, copiar y pegar, selección múltiple, miniaturas y vistas de iconos, lista y detalles. |
| **Ten tus carpetas a mano** | Coloca una carpeta en vivo sobre el escritorio. Navega por sus subcarpetas y ve los cambios al momento. |
| **Espacio cuando lo necesitas** | Contrae un grupo hasta su título. Pasa el cursor para expandirlo. Bloquea la distribución que te gusta. Haz doble clic en el escritorio para ocultar los grupos. Vuelve a hacer doble clic para recuperarlos. |
| **Siempre puedes volver atrás** | Instantáneas de distribución, copias de seguridad diarias, importación y exportación de la configuración e intercambio entre pantallas. |
| **Huella mínima** | Una aplicación nativa escrita en Rust que en reposo ocupa unos 40 MB de memoria (según el Administrador de tareas). El panel de Configuración en WebView2 se carga solo cuando hace falta. |

Las reglas de organización automática dejan los archivos en su ubicación original. Los movimientos
que inicias tú funcionan igual que en el Explorador de archivos.

[Explora la lista completa de funciones →](../FEATURES.md)

## Descarga PecoFence

<a href="https://apps.microsoft.com/detail/9MV6WG3XNWSX?mode=direct"><img src="https://get.microsoft.com/images/es%20dark.svg" alt="Consíguelo en Microsoft Store" width="200"></a>

La versión de Microsoft Store está firmada por Microsoft, se actualiza sola y nunca muestra el aviso de SmartScreen. La misma aplicación también está disponible como:

- **ZIP portátil**: descarga `pecofence-v<versión>-x64-portable.zip` desde la página de **Releases** de este repositorio, extrae el **ZIP completo** en una carpeta y ejecuta `pecofence.exe`.
- **Instalador**: `pecofence-v<versión>-x64-setup.exe`, en la misma página, instala PecoFence para tu usuario de Windows, con acceso en el menú Inicio y desinstalador.
- **winget**: `winget install DayuanJiang.PecoFence` instala la versión portátil y evita el aviso de SmartScreen.

<details>
<summary><strong>Requisitos, configuración y algunas notas útiles</strong></summary>

- Windows 11 22H2 o posterior. La Configuración necesita Microsoft Edge WebView2 Runtime.
- El primer inicio crea los grupos «Programas», «Carpetas», «Archivos y documentos» y «Escritorio» en el idioma que elijas. Los iconos del escritorio de Windows se restauran al salir.
- La configuración se guarda en `%APPDATA%\PecoFence\config.json`. Ejecuta la aplicación con `--portable` para guardarla en una carpeta `config` junto al ejecutable.
- El ZIP y el instalador no están firmados. Si Windows SmartScreen aparece en el primer inicio, elige **Más información → Ejecutar de todas formas**.

[Guía de la edición portátil](../PORTABLE.md) · [Guía de idiomas](../LOCALIZATION.md) · [Guía del instalador de Windows](../INSTALLER.md)

</details>

## Constrúyelo. Hazlo tuyo.

PecoFence tiene licencia Apache 2.0 y las contribuciones son bienvenidas: desde una traducción más
precisa hasta una interacción de escritorio mejor resuelta.

[Contribuir](../../CONTRIBUTING.md) · [Mejorar una traducción](../LOCALIZATION.md) · [Guía de desarrollo](../DEVELOPMENT.md)

---

**Hecho para un escritorio al que da gusto volver.**  
[Licencia Apache 2.0](../../LICENSE) · [Avisos de terceros](../../third_party/README.md)
