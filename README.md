# java-runner2026

> 中文版：[README_cn.md](README_cn.md)

Windows command-line tool: scan local JVM installations, let the user pick one, and run a configurable Java launch command in the **current terminal** with that JVM.

## Scan scope

- Registry (JavaSoft, Eclipse Adoptium, Microsoft JDK, Azul Zulu, etc.)
- Environment variables (names containing `JAVA` / `JDK` / `JRE`, plus `java.exe` entries on `PATH`)
- Common Java directories under `Program Files`
- File associations, `where java`, and typical install paths

## Build

```powershell
# Python dependencies (docs, icons, packaging)
pip install -r requirements.txt

# Multi-resolution icon (sources: 素材/icon_16x16.png … icon_256x256.png)
python scripts/build_icon.py

cargo build --release
```

## Documentation build

Sources (Markdown only under `doc/`): `doc/guide_cn.md`, `doc/guide_en.md`.

```powershell
# HTML, PDF, and index.html under dist/doc/ (PDF requires Microsoft Edge)
python scripts/build_docs.py

# PDF only
python scripts/build_doc_pdf.py
```

## Package for release

Merge the exe, `config/`, and HTML/PDF generated from `doc/guide_*.md` into `dist/java-runner2026/`:

```powershell
cargo build --release
python scripts/package_release.py   # includes doc build (dist/java-runner2026/doc/)
```

Release layout:

```
dist/java-runner2026/
  java-runner2026.exe      # Main program
  java-runner2026.toml     # Launch config (edit in place)
  config/                  # Config backup copy
  doc/                     # guide_cn/en HTML, PDF, index.html
```

Zip the entire `dist/java-runner2026` folder for distribution; **double-click `java-runner2026.exe` to run**. If the exe is placed in the project-root `release/` folder, the packaging script prefers that copy.

## Configuration

Resolved automatically in this order (override with `--config`):

1. `{exe directory}/java-runner2026.toml`
2. `{exe directory}/config/java-runner2026.toml`
3. `./java-runner2026.toml`
4. `./config/java-runner2026.toml`

```toml
# Optional: only list JVMs whose version output contains this keyword
# version_key = "1.8"

# Optional: auto-select a JVM whose path contains this fragment (skip prompt)
# java_home = "jdk1.8.0_271"

# Arguments after java.exe; default is equivalent to java -version
launch_args = ["-version"]

# Long args as a single-quoted literal (avoids TOML escapes like \logs):
# launch_args = '-Xmx512m -Xloggc:logs/gc.log -jar app.jar'
```

See [User Guide (Chinese)](doc/guide_cn.md) · [English Guide](doc/guide_en.md)

## Language

UI language follows the Windows user locale: `zh*` → Chinese, otherwise English. Override with `JAVA_RUNNER_LANG` (e.g. `zh-CN`, `en-US`).

## Usage

```powershell
# Scan → list JVMs → prompt → run configured launch command
.\target\release\java-runner2026.exe

# List JVMs only, do not launch
.\target\release\java-runner2026.exe --scan-only

# Use the 3rd JVM directly (skip interactive selection)
.\target\release\java-runner2026.exe --select 3

# JSON output for scripting (no selection or launch)
.\target\release\java-runner2026.exe --json

# Specify config file
.\target\release\java-runner2026.exe --config my.toml
```

## Interactive example

```
Found 6 JVM(s) (25 candidate(s), scan took 320ms)

[1] C:\Program Files\Java\jdk-19
    Version: 19.0.2
    ...

Select a JVM (enter number):
> 1

Using JVM: C:\Program Files\Java\jdk-19 (19.0.2)
Running: "C:\Program Files\Java\jdk-19\bin\java.exe" -version

java version "19.0.2" 2023-01-17
...
```

After selection, `JAVA_HOME` is set and its `bin` directory is prepended to `PATH` so the chosen Java is used.

When finished, the program waits for **any key** by default (so double-clicking the exe does not flash-close the window). Use `--no-pause` in scripts to skip the wait.

If no JVM is found: **No Java installation detected on this machine. Please install Java and run this program again.**

## Documentation

- [guide_cn.md](doc/guide_cn.md) / [guide_en.md](doc/guide_en.md) — user and implementation guides (Markdown sources)
- Run `python scripts/build_docs.py` to preview HTML/PDF under `dist/doc/`
