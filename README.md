# ChurchProjection Studio

A local-first Windows worship projection application built with Tauri 2, React, TypeScript, Tailwind CSS, Rust, and SQLite. It contains an operator dashboard and an independent audience display window. Runtime use does not require internet access.

## Included workflows

- Song CRUD with verses, choruses, bridges, intros, outros, per-section translations, text import, JSON export, and full-text search
- Bible JSON import, multiple versions/languages, reference and keyword search, and multilingual display
- Local JPG, PNG, MP4, and WebM media library with per-song and default backgrounds
- Separate movable display window, fullscreen, live preview, fade, overlay, typography controls, blank/logo/clear, and next/previous navigation
- SQLite migrations, automatic first-run database creation, sample song, settings persistence, and database backup
- Fully bundled React, Tailwind, icons, SQLite, and Rust dependencies; no CDN or remote runtime request

## Prerequisites on Windows

1. Install Node.js 20 or newer.
2. Install Rust with `winget install Rustlang.Rustup`, then open a new terminal and run `rustup default stable-msvc`.
3. Install Visual Studio Build Tools 2022 with **Desktop development with C++** and a Windows SDK.
4. Install Microsoft Edge WebView2 Runtime (normally already present on Windows 10/11).

See the official Tauri 2 Windows prerequisites if a native build tool is missing: https://v2.tauri.app/start/prerequisites/

PowerShell may block `npm.ps1`. Use `npm.cmd` in that case, as shown below.

## Install and run

```powershell
cd C:\xampp\htdocs\project
npm.cmd install
npm.cmd run tauri dev
```

The dashboard opens with a sample song. Select **Open projection**, drag that window to the second monitor, then use **Fullscreen** in Projection Control. Use the Songs or Bible pages to send content live. Arrow keys change slides, `B` blanks the output, and `Esc` clears text.

`npm.cmd run dev` opens only the browser UI and is useful for visual work, but native database/dialog/window features require `npm.cmd run tauri dev`.

## Build Windows installers

```powershell
npm.cmd run check
npm.cmd run tauri build
```

Successful builds are placed under (the version number changes per release):

- `src-tauri\target\release\bundle\nsis\ChurchProjection Studio_1.0.1_x64-setup.exe`
- `src-tauri\target\release\bundle\msi\ChurchProjection Studio_1.0.1_x64_en-US.msi`

### Build an installable update

Keep the identifier `org.churchprojection.studio` unchanged and increase the semantic version for every release. The release script synchronizes the version in npm, Cargo, Tauri, and the Settings screen, then builds a new NSIS installer:

```powershell
npm.cmd run release:update -- -Version 1.0.2
```

Close ChurchProjection Studio and run the new setup file. NSIS upgrades the existing installation while the SQLite database and imported media under `%APPDATA%\org.churchprojection.studio` remain intact. This update process is completely offline once the installer has been copied to the church computer.

Starting with version 1.0.4, operators do not need to run future setup files manually. Copy the newer `ChurchProjection Studio_<version>_x64-setup.exe` to USB, Downloads, or a shared folder, then open **Settings > Install application update > Choose and install update**. Select the file and confirm. The application closes, installs the update silently, and preserves all local content.

The first upgrade from an older version to 1.0.4 must still be run manually because older installed binaries do not contain the update button.

The installer and installed application work offline. Cargo/npm need network access only when dependencies are first downloaded or updated.

## Import formats

### Songs

Use UTF-8 `.txt` files. Separate sections with a blank line and optionally begin a section with `Verse 1:`, `Chorus:`, `Bridge:`, `Intro:`, or `Outro:`. See `examples/song-sample.txt`.

### Bibles

Bible import uses UTF-8 JSON. Each `chapters` item is an array of verse strings; array positions become one-based chapter and verse numbers. See `examples/bible-sample.json`. Import only texts you are licensed to use publicly.

## Local data

The database and copied media are stored in the Tauri application data directory, normally:

```text
%APPDATA%\org.churchprojection.studio\
```

Use **Settings > Export database** before moving installations or making major library changes. Imported media is copied into the app data directory, so removing the source file does not affect projection.

## Project structure

```text
src/                       React operator and projection interfaces
  App.tsx                  Dashboard and all management workflows
  ProjectionWindow.tsx     Independent audience display
  api.ts                   Typed Tauri command boundary
  types.ts                 Shared application contracts
src-tauri/
  src/lib.rs               SQLite, imports, media, backup, and window commands
  src/schema.sql            Idempotent database migration/schema
  capabilities/default.json Tauri 2 permissions
  tauri.conf.json           Windows app and installer configuration
examples/                   Bible and song import samples
```

## Extension notes

Database access stays behind Tauri commands. Add migrations with a new version in `schema_migrations`, preserve foreign keys, and expose new commands through `api.ts`. The display is driven by the typed `projection:update` event, which keeps new slide types independent from database storage.
