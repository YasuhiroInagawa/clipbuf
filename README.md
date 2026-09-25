# clipbuf

**See it, stop it, fix it** — before you paste.

*[日本語版はこちら](README.ja.md)*

clipbuf picks up whatever you copy in other applications and keeps the last N items, one line
each. It marks the characters you cannot otherwise see — spaces, ideographic spaces, tabs, line
breaks, non-breaking spaces, zero-width characters — with their own symbol and colour, and warns
about formatting, platform-dependent characters and mixed line endings. Click a row and clipbuf
writes it back to the clipboard with the clean-ups you chose applied. The stored item itself is
never altered.

Windows, macOS and Linux. Free and open source (MIT).

## What it does

- **Captures automatically** — watches for copies in other applications, keeping the most recent
  ones in a FIFO buffer (20 by default)
- **Makes the invisible visible** — spaces, tabs, line breaks, NBSP and zero-width characters get
  a symbol and a colour of their own; CRLF, LF and CR are told apart
- **Warns** — formatting present, leading or trailing whitespace, tabs, platform-dependent
  characters, control characters, mixed line endings
- **Transfers cleanly** — strip formatting, drop or replace line breaks, trim, tabs to spaces,
  ideographic spaces to ordinary ones, each a toggle
- **Stays out of the way** — a small always-on-top window, a global hotkey (`Alt+Shift+V` by
  default) and a tray icon

## What it never does with what you copy

- Keeps captured items **in memory only**. Nothing is written to disk, and everything is
  discarded when clipbuf exits
- Never sends their contents over the network. The **update check is the only network access**
- Never writes their contents to a log
- Ignores anything marked as not-for-monitoring by password managers and the like

## Installing

Grab the build for your system from [Releases](https://github.com/YasuhiroInagawa/clipbuf/releases).
Every release also carries `SHA256SUMS.txt` for all files and `THIRD-PARTY.md`, the licences of
everything clipbuf is built on.

### Windows

Run `clipbuf_x.y.z_x64-setup.exe`, the NSIS installer.

**Windows will show "Windows protected your PC".** clipbuf is not code signed: a certificate
costs more per year than a free tool from one person can justify. To continue:

1. Click **More info** in the blue dialog
2. Click the **Run anyway** button that appears

If you would rather check first, compare the download against `SHA256SUMS.txt`:

```powershell
Get-FileHash .\clipbuf_x.y.z_x64-setup.exe -Algorithm SHA256
```

### macOS

Open `clipbuf_x.y.z_universal.dmg` and drag clipbuf to Applications. It is a universal binary, so
it runs natively on both Apple Silicon and Intel. The app is signed and notarised, so it starts
without a Gatekeeper warning.

**If nothing is captured, macOS is withholding the clipboard.** clipbuf then says so, and you can
allow it:

1. System Settings › Privacy & Security › **Pasteboard**
2. Turn clipbuf on

clipbuf lives in the menu bar and stays out of the Dock.

### Linux

AppImage, deb and rpm are provided.

```bash
chmod +x clipbuf_x.y.z_amd64.AppImage && ./clipbuf_x.y.z_amd64.AppImage
```

```bash
sudo dpkg -i clipbuf_x.y.z_amd64.deb
```

**On Wayland, capturing depends on your compositor.** clipbuf reads another application's
clipboard, which a compositor has to allow for a window that is not focused.

| Environment | Capture |
| --- | --- |
| X11 | Everything |
| KDE Plasma (Wayland) | Everything, through `wlr-data-control` |
| Sway and other wlroots compositors | Same |
| GNOME (Wayland) | **Nothing but XWayland.** GNOME does not implement `wlr-data-control` |

Under GNOME on Wayland, clipbuf says so in a banner at the top of the window. Log in to an X11
session instead if you need to capture from every application.

## Updates

clipbuf checks for a new version at startup and offers it in a banner. Accept and it downloads,
installs and asks you to restart; decline and it keeps running the version you have. Updates are
signed, and one whose signature does not verify is not installed.

## Settings

From the tray icon, or the menu bar on macOS: how many items to keep, the global hotkey, tab
width, polling interval (macOS and Wayland only), start at login, whether the preview wraps long
lines, and the interface language (English, Japanese, or follow the system).

## Getting in touch

Questions and bug reports belong in
[Issues](https://github.com/YasuhiroInagawa/clipbuf/issues).

## Developing

```bash
npm install && npm run tauri dev
```

```bash
npm test && npm run check && npm run lint
```

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

`npm run licenses` refreshes the third-party licence list. The manual checks that the automated
suite cannot cover are in [doc/platform-checklist.md](doc/platform-checklist.md).

## Licence

MIT, see [LICENSE](LICENSE). The licences of everything clipbuf depends on are collected in
[THIRD-PARTY.md](THIRD-PARTY.md).
