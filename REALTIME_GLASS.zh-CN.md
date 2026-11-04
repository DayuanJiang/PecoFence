# 实时液态玻璃与 Wallpaper Engine 场景壁纸

[English](REALTIME_GLASS.md)

本分支为 PecoFence 增加一个可选的实时 GPU 背景路径，让现有 Liquid Glass 分组框持续更新模糊、颜色和折射效果。功能默认关闭，刷新上限与效果参数可以在设置页调整。

当前已在一台 Windows 设备上验证 Wallpaper Engine（WE）的场景壁纸。本文区分实现方式、已有验证和仍待测试的范围，供使用、复现和继续开发参考。

## 要解决的问题

原有动态壁纸支持采用定期截图：壁纸变化、返回桌面等事件触发刷新，可见时还会定时更新。它并非逐帧背景，因此视频壁纸可以显示，也不代表分组框中的画面会实时变化。

另外，WE 某些场景或网页播放器不提供原有 `PrintWindow` 路径需要的重定向位图，旧路径可能返回黑屏或旧画面；PF 会用 `notCapturable` 表示该截图路径不可用。

本实现通过 Windows Graphics Capture（WGC）读取桌面宿主的合成画面，将 GPU 纹理接入现有玻璃渲染。它没有调用 WE 私有接口，也不解析或重放 WE 的场景文件。

## 画面从哪里来

```text
WE 场景渲染
    ↓
Windows 桌面宿主的合成画面（当前目标：Progman）
    ↓ Windows Graphics Capture，有界帧池，只消费最新帧
PF 自有的持久 D3D11 纹理
    ↓ Direct2D 下采样、模糊、饱和度、白色薄底色
所有分组框共享的背景材质
    ↓ 按各框屏幕坐标采样，叠加边缘折射与高光
分组框显示；图标内容保持在独立视觉层
```

同一来源的纹理和材质会复用，正常逐帧更新不重新创建整套材质；多个框共享一次背景更新。实时背景的生产路径不把背景像素读回 CPU，不编码 PNG/JPEG/视频，也不经由磁盘把画面传给 PF。

两个 WGC 帧缓冲供给现有 D3D11 设备上的一张持久纹理。使用同一设备的 D2D 上下文每次更新只处理一次共享材质，各框按物理桌面坐标采样，并使用已有的 GPU 透镜和边缘效果。网格图标表面保持缓存；列表／详细视图中使用玻璃的标题区域按需刷新。正常逐帧更新不创建新的 GPU 设备。

分组框是独立的顶层窗口，不属于当前采集的桌面宿主，因此该路径不会采入分组框自身，也无需通过排除捕获来隐藏分组框；正常截图仍可看到分组框。当前没有启用整屏／显示器采集作为回退。

这里捕获的是桌面宿主，**不是完整的任意前台应用滤镜**。例如将分组框浮到其他应用前面时，不能据此认为它会折射该应用的内容。当前画面来源、窗口层级和系统合成方式仍然影响结果。

新增采集路径不向 WE 或 Explorer 注入代码，不修改它们的窗口父子关系，也不对它们打补丁。它不重启 Explorer、不修改 WE 播放设置，也不改变系统全局定时器分辨率。刷新时钟使用本进程自有的高分辨率单次可等待定时器，最多保留一个待处理的 UI 刷新通知，不调用 `timeBeginPeriod`；不可用时回退到普通定时器。WGC 捕获边框和访问策略仍由系统能力和权限决定。

主要实现入口：

| 位置 | 职责 |
| --- | --- |
| `crates/render/src/live_capture.rs` | WGC 会话、有界帧池、持久 GPU 纹理、来源关闭与尺寸变化 |
| `crates/render/src/realtime_material.rs` | 共享背景的下采样、模糊与颜色处理 |
| `crates/render/src/gpu_glass.rs` | 分组框和小控件的透镜、输入失效及材质复用 |
| `crates/app/src/app/realtime_glass.rs` | 刷新调度、遮挡暂停、重试、设备恢复与运行诊断 |
| `crates/platform/src/realtime_clock.rs` | 进程内刷新时钟，丢弃过期节拍，限制待处理通知数量 |
| `crates/core/src/model.rs`、`ui/settings.html` | 参数模型、校验和设置页 |

## 配置与使用

使用实验构建时，在“常规”中选择“液态玻璃”，再开启“实时液态玻璃（实验版）”。已有配置缺少新字段时保持关闭；关闭功能后继续使用原来的背景路径。

下表的键属于 `config.json` 中的 `settings.realtimeGlass` 对象；CLI 使用表中最后一列的路径。

| 字段 | 范围或选项 | 默认值 | CLI 路径 |
| --- | --- | --- | --- |
| `enabled` | `true` / `false` | `false` | `realtimeGlass.enabled` |
| `fps` | 整数 5–60 | `30` | `realtimeGlass.fps` |
| `quality` | `performance` / `balanced` / `quality` | `balanced` | `realtimeGlass.quality` |
| `blur` | 0–30，原始画面的物理像素 | `8` | `realtimeGlass.blur` |
| `refraction` | 0–2，0 关闭折射 | `1` | `realtimeGlass.refraction` |
| `saturation` | 0–2；0 为灰度，1 保留原色 | `1.05` | `realtimeGlass.saturation` |
| `tint` | 0–0.5，白色薄底色强度 | `0.08` | `realtimeGlass.tint` |
| `pauseWhenCovered` | `true` / `false` | `true` | `realtimeGlass.pauseWhenCovered` |

浮点参数必须为有限数。`blur` 的单位是物理像素，不随 Windows 的 DIP 缩放倍率增加。`tint` 是白色薄底色，不代表自动选择深色或浅色的底色。

采样质量决定背景处理中间纹理的宽高：省电档各为原来的 1/4，均衡档各为 1/2，精细档保留原尺寸。低档位减少部分材质处理和纹理开销，**WGC 原始帧池与输入复制仍使用原始分辨率**。FPS 是刷新上限，实际速度也受壁纸播放、系统调度和 GPU 影响。

在确认目标 PF 实例正在运行后，可通过随构建提供的 CLI 修改设置。例如在仓库根目录执行：

```powershell
.\target\release\pecofence-cli.exe settings get realtimeGlass
.\target\release\pecofence-cli.exe settings set themeStyle liquidGlass
.\target\release\pecofence-cli.exe settings set realtimeGlass.fps 30
.\target\release\pecofence-cli.exe settings set realtimeGlass.quality balanced
.\target\release\pecofence-cli.exe settings set realtimeGlass.enabled true
```

关闭实时功能：

```powershell
.\target\release\pecofence-cli.exe settings set realtimeGlass.enabled false
```

`pauseWhenCovered` 开启时会检查实际位于分组框上方的覆盖窗口。所有分组框均无需更新时，采集会话和高频刷新时钟暂停；恢复可见后重新建立。窗口层级检查约每 250 毫秒一次，因此暂停与恢复不是瞬时发生。当前检查依据单个上方窗口是否完全覆盖一个框，不能把多个窗口拼接遮挡等同于已验证的完整遮挡分析。

来源关闭、初始帧等待超时、采集或 GPU 错误会进入重试或原有背景路径；重试间隔有上限，不进行无界快速重试，设备丢失时沿用既有的 GPU 恢复路径。`active` 和帧数增加表示采集与更新正在进行，单凭这些值不能证明每帧内容都正确，也不能保证自动识别所有黑屏。

## 构建与测试

以下命令在仓库根目录执行。开发环境与上游一致：Windows/MSVC 工具链、C++ Build Tools、Windows SDK，以及设置页需要的 WebView2 Runtime。当前 `Cargo.toml` 要求 Rust 1.95 或更新版本，本次验证使用 Rust 1.95.0；Python 用于资源检查，Node.js 用于设置页测试。完整说明见 [开发指南](docs/DEVELOPMENT.md)。

```powershell
cargo build --locked --release -p pecofence -p pecofence-watchdog
cargo build --locked --release -p pecofence-cli
Copy-Item -LiteralPath third_party\webview2\WebView2Loader.x64.dll -Destination target\release\WebView2Loader.dll
```

`WebView2Loader.dll` 需要位于程序旁，即使此次不打开设置页。分发或试用时也保留 watchdog、许可证和第三方声明。正常退出另一份桌面管理实例、备份配置后，再从独立目录试用便携构建。需要沿用配置时，应在目标实例停止后，将导出的配置复制到该独立目录的 `config/config.json`。保留正常的单实例保护，同一时间只由一个 PF 实例管理这份桌面。

下面的示例从构建输出目录启动；若已复制到独立便携目录，则运行该目录中的同名程序：

```powershell
.\target\release\pecofence.exe --portable
```

便携配置保存在程序旁的 `config` 目录。`PECOFENCE_RUNTIME_DIR` 可设为绝对目录，将此构建的日志和 WebView2 运行数据放到独立位置，无需覆盖 `LOCALAPPDATA`；它不改变便携配置目录。运行时 `status.runtimeLogDir` 可供 CLI 的 `paths` / `log` 使用。命名实例可配合开发指南进行隔离测试，但实例名本身不为 `--portable` 配置创建独立目录。

本功能相关的 Rust 检查命令：

```powershell
cargo fmt --all --check
cargo test --locked -p pecofence-core --features schema
cargo test --locked -p pecofence-render --lib
cargo test --locked -p pecofence --bin pecofence
cargo test --locked -p pecofence-cli --bin pecofence-cli
cargo test --locked -p pecofence-ipc
```

贡献前还应按照 [CONTRIBUTING.md](CONTRIBUTING.md) 完成仓库要求的检查，包括：

```powershell
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python scripts/check-locales.py
python scripts/check-readme-translations.py
node scripts/test-settings-ui.mjs
```

设置页脚本会提供本地测试页面，需打开页面并检查结果；启动测试服务并不等于浏览器测试已经通过。部分 Windows 平台测试会调用真实系统 API，需要桌面会话。

生产代码的 Windows 绑定可由 `cargo run -p tool_bindgen` 重新生成，本分支已把 WGC 绑定加入默认生成列表及 CI 一致性检查。可选原生背景探针另用 `cargo run -p tool_bindgen -- tools/bindgen/host-backdrop.txt` 生成。

### 首次源码迁入校验

2026-11-04 在目标工作区、Rust 1.95.0 下重新执行：

- `cargo fmt --all -- --check` 和严格的全工作区／全目标 Clippy 检查通过。
- `cargo test --workspace --locked`：**524 项通过，1 项原有测试忽略**。
- 浏览器设置回归：**80 项通过，0 项失败**；使用模拟桥接，不修改运行中 PF 的设置。
- 11 种语言的 616 条消息覆盖检查、11 份 README 翻译检查通过。
- 5 套生产 Windows 绑定重新生成后与已迁入文件完全一致。

为适配严格检查，还对生成绑定模块局部豁免了系统类型名／生成投影的两项 lint，并将既有临时文件规则中的布尔判断作等价简化；未改变对应规则的行为。这里的校验结果与下文先前实验构建的视觉／性能实测分开记录。

### 合入上游 0.2.5 后的校验

2026-11-04 将上游提交 `ef888bf878f7cb634971123eae1dff931e537b30` 合入本分支后，在 Rust 1.95.0 下重新执行：

- `cargo fmt --all -- --check` 和严格的全工作区／全目标 Clippy 检查通过。
- `cargo test --workspace --locked`：**529 项通过，1 项原有测试忽略**。
- 浏览器设置回归：**95 项通过，0 项失败**；使用模拟桥接。
- 11 种语言的 633 条消息覆盖检查、11 份 README 翻译检查通过。
- 5 套生产 Windows 绑定重新生成后与合并后的文件完全一致。

此次未启动合并后的桌面程序进行新的视觉或性能实测。下文的既有测量仍对应先前的实验构建。

## 有界诊断

独立探针可以检查桌面宿主画面是否持续变化，无需启动另一份 PF 桌面管理实例：

```powershell
cargo build --locked --release -p pecofence-render --example realtime_glass_probe
.\target\release\examples\realtime_glass_probe.exe --exit-after 20
```

探针会创建一个可见的小型预览窗，默认以 30 FPS 上限、均衡质量运行，到期退出并将 JSON 汇总写到标准输出。`--exit-after` 在此探针中的单位为**秒**，允许 1–300，默认 20；主程序同名参数的单位则为毫秒。探针累计出现三次错误后也会结束运行。

探针支持 `--source-hwnd` 和 `--inspectable`，默认测试应保留桌面宿主来源。它不修改桌面图标、壁纸或 PF 配置，也不保存截图和视频。

为了区分“帧计数增长”和“像素实际变化”，**仅探针**每秒读取九块 16×16 像素区域，约 9 KiB，计算非黑采样、不同哈希和相邻变化次数。探针 JSON 的 `cpuReadback: true` 指这些诊断采样；正式程序的实时生产路径仍无 CPU 像素回读。小区域变化也不能代表完整图像正确或没有视觉瑕疵。

源码还保留了 `crates/render/examples/host_backdrop_probe.rs`，用于探索原生背景材质替代方案，以及 `crates/render/examples/occlusion_probe.rs`，用于创建 8 秒后自动关闭的不透明遮挡测试窗。这些是独立诊断工具，不会作为主程序的日常采集路径运行。

运行中的 PF 可通过以下命令查看刷新状态与日志位置：

```powershell
.\target\release\pecofence-cli.exe status --fields realtimeGlass,runtimeLogDir
.\target\release\pecofence-cli.exe paths
```

`realtimeGlass` 包括 `state`、`frames`、`paneDraws`、`averageFps`、`lastFrameAgeMs`、`maxTickMs`、`error` 和 `cpuReadback` 等字段。会话重建时计数重新开始；暂停状态下的平均 FPS 不宜作为运行性能。`maxTickMs` 包括会话初始化与失败路径的 UI tick 时间，不等于 GPU 每帧耗时或端到端延迟。

## 已有本机短测

以下数字来自 **2026-11-04 的先前实验构建记录**，环境为 Windows build 26300、2560×1440 单屏、SDR、Ryzen 7 9700X（16 逻辑处理器）和 RTX 5070，使用当时正在运行的 WE 场景壁纸。这是本机短测例，不是跨设备性能承诺，也不是这次源码迁入后的重新测试结果。

| 短测场景 | 采样秒数 | 会话平均 FPS | PF CPU | 工作集 MiB | 专用 GPU 内存 MiB | PF 3D 引擎采样均值 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 原版静态背景路径 | 11.2 | — | 0.00% | 15.3 | 116.6 | 0.00% |
| 15 FPS／省电，持续更新压力测试 | 11.4 | 14.9 | 0.22% | 105.0 | 204.3 | 0.00% |
| 30 FPS／均衡，持续更新压力测试 | 21.2 | 29.8 | 0.30% | 104.8 | 184.9 | 1.00% |
| 60 FPS／精细，持续更新压力测试 | 11.1 | 59.3 | 0.53% | 105.2 | 233.0 | 1.67% |
| 30 FPS／均衡，确认 WE 播放后的另一轮 | 21.1 | 28.7 | 0.35% | 110.8 | 353.4 | 1.00% |
| 最终构建，30 FPS／均衡，默认遮挡暂停策略 | 11.1 | 29.3 | 0.23% | 102.8 | 169.7 | 0.00% |

这些结果的统计口径是：

- CPU 为 PF 主进程 CPU 时间增量除以墙钟时间和逻辑处理器数。表格不包含 WE、DWM 或 WebView2 子进程的额外开销。
- GPU 3D 为三次采样中，最忙的该进程 3D 引擎实例的均值，没有累加不同引擎；不是整机 GPU 负载或功耗。短时读到 0% 不代表零消耗。
- FPS 是采样结束时该次采集会话的累计平均值，包括初始化；不是该采样时间窗单独计算的帧数增量，也不是内容到屏幕的延迟。
- 压力测试关闭了遮挡暂停；最后一轮恢复默认策略，可见更新范围不同。切换质量、重建会话和设置窗口还会改变驱动与合成缓存，不能从各行推导质量档位的固定显存预算。
- 此次 PF 专用 GPU 内存的轮次均值约 170–353 MiB，原版约 117 MiB。工作集受 Windows 回收影响。未测墙插功耗、整机能耗或长时间显存趋势。

先前验证还包括：独立动态探针运行 60 秒，1,784 帧、约 29.7 FPS；59 次非黑背景采样得到 59 个不同哈希，58 次相邻比较均变化，错误为 0。另进行了短时全遮挡暂停／恢复、四次开关循环和正常退出／恢复原版测试；短时没有观察到句柄和线程单调累加。这些结果不能替代长期稳定性验证。

先前实验构建的 Core 91、Render 72、App 235、CLI 49、IPC 20 项测试通过，合计 **467 项**，另有一项原有 App 测试忽略。467 这一计数仅对应先前实验构建；源码迁入及合入上游 0.2.5 后重新执行的完整测试结果见上文各校验小节。先前实验版的 release 构建也已通过。

## 兼容边界与后续验证

目前实际视觉和性能验证仅覆盖上述单屏 SDR 环境中的 WE 场景壁纸。原版对视频壁纸可截图的行为，不等于已经验证本分支对视频、网页和其他动态壁纸软件的实时兼容性。

多屏、混合 DPI、HDR、其他 Windows build、其他 GPU／驱动、WE 网页与视频壁纸、休眠唤醒和长时间播放仍需单独复现。代码包含尺寸变化与设备恢复处理，也不等于这些场景已经全部通过实测。当前实时纹理采用 8-bit BGRA，不能宣称保留 HDR 高光。

WGC 不可用、软件光栅化回退或当前材质不是 Liquid Glass 时，实时功能不会按正常 GPU 路径运行。部分较旧的 Windows 版本不支持可选的采集生产端帧率限制属性，此时消费端的刷新上限仍然生效。WE 自己的暂停／停止规则仍生效，PF 不会强制它继续渲染。共享桌面宿主的范围和合成行为随系统而变，仍可能包含桌面图标层等内容，需要在具体环境中检查来源是否符合预期。

复现报告应包含 Windows build、显示器拓扑／缩放／HDR、GPU／驱动、壁纸类型、刷新与质量档位、是否遮挡，以及有界探针和状态汇总。提交前检查个人路径、窗口标识和日志内容；用户配置、截图、原始本机证据和机器专用部署脚本不作为公开源码的一部分。

## 来源、版本与许可证

上游项目为 [DayuanJiang/PecoFence](https://github.com/DayuanJiang/PecoFence)。本实验最初基于上游提交 `40b2acf6dcc349b477b426862acdf78900a79768`，该基础版本为 `0.2.4`，实验包版本为 `0.2.4-liveglass.1`。现已合入上游 `0.2.5`，对应提交 `ef888bf878f7cb634971123eae1dff931e537b30`。`Cargo.toml` 中的包版本随上游采用 `0.2.5`；实时玻璃仍是本功能分支上的可选扩展，未表示上游已经发布该功能。

迁入时保留上游完整工程、原有功能和 `vendor` 依赖，只在该基础上加入实时玻璃所需的实现及设置。许可证继续沿用 [Apache License 2.0](LICENSE)；原作者署名、第三方声明和 vendored 依赖各自的许可文件均应保留，见 [第三方依赖说明](third_party/README.md)。
