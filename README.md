# java-runner2026

Windows 命令行工具：扫描本机 JVM 安装位置，提示用户选择一个，并在**当前终端**用该 JVM 运行可配置的 Java 启动命令。

## 扫描范围

- 注册表（JavaSoft、Eclipse Adoptium、Microsoft JDK、Azul Zulu 等）
- 环境变量（名称含 `JAVA` / `JDK` / `JRE` 的变量，以及 `PATH` 中的 `java.exe`）
- `Program Files` 常见 Java 目录
- 文件关联、 `where java` 、常见安装路径

## 构建

```powershell
# Python 脚本依赖（文档/图标/打包）
pip install -r requirements.txt

# 生成多分辨率图标（源文件：素材/icon_16x16.png … icon_256x256.png）
python scripts/build_icon.py

cargo build --release
```

## 文档构建

源文件（`doc/` 目录仅保留 Markdown）：`doc/guide_cn.md`、`doc/guide_en.md`。

```powershell
# 生成 dist/doc/ 下 HTML、PDF、index.html（需 Microsoft Edge 生成 PDF）
python scripts/build_docs.py

# 仅生成 PDF
python scripts/build_doc_pdf.py
```

## 打包发布（开箱即用）

将 exe、`config/` 与由 `doc/guide_*.md` 生成的 HTML/PDF 合并到 `dist/java-runner2026/`：

```powershell
cargo build --release
python scripts/package_release.py   # 内含文档构建（dist/java-runner2026/doc/）
```

发布目录结构：

```
dist/java-runner2026/
  java-runner2026.exe      # 主程序
  java-runner2026.toml     # 启动配置（可直接编辑）
  config/              # 配置备份
  doc/                 # guide_cn/en 的 HTML、PDF、index.html（无 README）
```

可将整个 `dist/java-runner2026` 文件夹压缩分发给用户；**直接双击 `java-runner2026.exe` 即可运行**。若已将 exe 放入项目根目录的 `release/` 文件夹，打包脚本会优先使用该路径。

## 配置文件

按以下优先级自动查找（也可用 `--config` 指定路径）：

1. `{exe 目录}/java-runner2026.toml`
2. `{exe 目录}/config/java-runner2026.toml`
3. `./java-runner2026.toml`
4. `./config/java-runner2026.toml`

```toml
# 可选：仅显示版本输出含该关键字的 JVM
# version_key = "1.8"

# 可选：自动选用路径含该片段的 JVM（跳过交互）
# java_home = "jdk1.8.0_271"

# java.exe 之后的参数，默认等效于 java -version
launch_args = ["-version"]

# 长参数可用单引号字面量（避免 \logs 等 TOML 转义问题）:
# launch_args = '-Xmx512m -Xloggc:logs/gc.log -jar app.jar'
```

详见 [使用与实现指南（中文）](doc/guide_cn.md) · [English Guide](doc/guide_en.md)

## 语言

界面语言随 Windows 用户区域自动选择：`zh*` 为中文，否则为英文。可用环境变量 `JAVA_RUNNER_LANG`（如 `zh-CN`、`en-US`）覆盖。

## 使用

```powershell
# 扫描 → 列出 JVM → 提示选择 → 运行配置中的启动命令
.\target\release\java-runner2026.exe

# 仅扫描列表，不启动
.\target\release\java-runner2026.exe --scan-only

# 直接选择第 3 个 JVM 并启动（跳过交互）
.\target\release\java-runner2026.exe --select 3

# JSON 输出（供脚本使用，不进入选择/启动）
.\target\release\java-runner2026.exe --json

# 指定配置文件
.\target\release\java-runner2026.exe --config my.toml
```

## 交互示例

```
共发现 6 个 JVM（候选 25 条，扫描耗时 320ms）

[1] C:\Program Files\Java\jdk-19
    版本: 19.0.2
    ...

请选择要使用的 JVM（输入序号）:
> 1

使用 JVM: C:\Program Files\Java\jdk-19 (19.0.2)
执行命令: "C:\Program Files\Java\jdk-19\bin\java.exe" -version

java version "19.0.2" 2023-01-17
...
```

选中 JVM 后会设置 `JAVA_HOME`，并将其 `bin` 目录置于 `PATH` 最前，确保调用的是所选 Java。

执行完成后默认会提示 **按任意键退出**（避免双击 exe 时窗口闪退）。在终端脚本中可加 `--no-pause` 跳过等待。

未检测到 JVM 时显示：**未能检测到该机器安装的java环境，请安装后再执行本程序**

## 文档

- [guide_cn.md](doc/guide_cn.md) / [guide_en.md](doc/guide_en.md) — 使用与实现指南（源 Markdown）
- 运行 `python scripts/build_docs.py` 后在 `dist/doc/` 查看 HTML/PDF
