# AGENTS.md

本文件由 prux 在启动时自动加载（项目级上下文）。动手前至少读「分层与硬约定」与「陷阱」两节。

## 这个仓是什么

[KK_UI](https://gitee.com/keysking/kk_ui)（C，单色菜单 UI）的 Rust 移植。单 crate、`#![no_std]`、
无外部依赖、无 `alloc`：库只产出 `width × ceil(height/8)` 字节的 1 bit 帧缓冲，显示驱动与业务都在
调用方。默认构建（不带任何 feature）就是纯 `core`。

上游 C 版的 `KK_UI_ENABLE_*` 编译开关没有移植：所有页面与弹窗都编译，未用到的部分交给链接器剥掉。
异步 IT/DMA 刷新也没移植：渲染结果直接落在 `Frame`，提交由应用阻塞完成。

## 仓库地图

| 位置 | 内容 |
|---|---|
| `src/lib.rs` | 唯一公开面：模块声明、`pub use`、四个批量声明宏 |
| `src/gfx.rs` | 页式帧缓冲、半开裁剪窗口、`DrawMode`、图元与文字绘制 |
| `src/font.rs` | 全局字体缓存（`set_font` / `Glyphs`）与字形查找 |
| `src/model.rs` | 应用描述表：路由、页面、绑定、`Shared` / `Text` 共享单元 |
| `src/ui.rs` | 门面：模块声明、再导出、`KkUi` 本体 |
| `src/ui/*` | `Runtime` 按能力一个文件，各自给 `Runtime` 加 `impl`（见下） |
| `src/widgets/*` | 复用组件；`pages/` 三个页面、`dialog/` 弹窗与 Toast |
| `src/cjk.rs` | 内嵌字库（仅 `cjk-fonts`） |
| `src/sim.rs` | 宿主机 eframe 模拟器（仅 `sim`，唯一用 `std` 的地方） |
| `bin/kkui-tool.rs` | 资源工具：`font` 采字符生成字库、`icon` 图片转 XBM（仅 `tool`） |
| `build.rs` + `build/bdf.rs` | 编译期从 `assets/fonts/unifont.bdf` 抠字模；`build/bdf.rs` 与工具共用（`include!` + `#[path]`），所以它内部不写 `use`、路径一律写全 |
| `assets/fonts/unifont.bdf` | 约 9 MB 的 Unifont 副本，入库，构建不依赖本机字体 |
| `examples/sim/` | 完整示例：`main.rs` 接线 + `app.rs` 门面 + `app/{tables,handlers,autoplay,wave,math}.rs` |
| `tests/` | 按主题分文件：`frame` / `text` / `cjk` / `custom` / `dialog` / `menu_scroll` / `widgets` / `sim` |
| `changelog/v<version>.md` | 中文发布说明，按版本一文件 |
| `lilt/` | **忽略的嵌套 Go 仓**（TinyGo GUI 框架，动画与布局的参考实现）。只读，不改、不提交 |

## 分层与依赖方向

```text
应用（本 crate 之外） → KkUi / CustomPageHandler → Runtime → widgets → gfx → font
```

- `gfx` 不依赖 `widgets` / `ui` / `model`；`widgets` 只向下依赖 `gfx`（`pages`/`dialog` 例外，
  它们要用 `Runtime`）。同 crate 里跨层互调照样能编译，没有机器检查会拦你，只有人眼会翻车。
- `src/ui/` 的分工是**一个能力一个文件**，都通过 `impl Runtime` 挂上去：`status` 结果与错误、
  `timing` 帧间隔常量、`anim` 缓动与定点换算、`state` 页面状态与各动画结构、`input` 按键消抖、
  `nav` 导航栈与转场、`deferred` 弹窗入口与延迟请求、`events` 事件队列、`validate` 描述表校验、
  `custom` 自绘页回调分发、`scene` 整屏绘制。加东西先找对应文件，别往 `ui.rs` 或 `runtime.rs` 里塞。
- `ui.rs` 只做门面：模块声明、再导出、`KkUi` 的转发方法。`Runtime` 的字段与基础操作在
  `ui/runtime.rs`。

## 硬约定

- **应用描述表是编译期常量**：路由、页面、绑定、文案全在 `static` 里，`Runtime` 只读；可变状态
  只能走 `Shared<T>` / `Text`。所以新增页面＝声明 `static ROUTE_*` 再挂进 `App.routes`，
  **不要**去加第二张需要对齐的表（路由表第 `n` 项就是页面号 `n + 1`）。
- **`Shared` / `Text` 的 `unsafe impl Sync` 靠单核假设**：只在 UI 主循环与其回调里顺序访问，
  不跨核、不进中断。想加别的 `unsafe impl Sync` 先想清楚这条前提还成不成立。
- **库本体不许出现浮点**：`no_std` 没有 `libm`。动画用 `src/ui/spring.rs` 的 Q8 定点弹簧、
  缓动用 `anim.rs` 的 `ease_q12` / `lerp_q12`；曲线用查表加整数缩放（见 `examples/sim/app/wave.rs`）。
  `f32` 只允许出现在 `src/sim.rs`（egui 侧）。
- **批量声明走库内宏**：`event_ids!`（按声明顺序生成从 1 开始的事件号）、`shared!`、`texts!`、
  `menu_states!`（定长状态数组，长度由递归 `@len` 数出来）、`icons!`（`include_bytes!` 路径相对
  调用该宏的文件解析）。不要在示例或测试里再手写等价的静态声明。
- **`enum` ↔ 字符串转换用 `strum`**：不要手写 `as_str` / `from_str`。注意本 crate 默认无外部依赖，
  引入 `strum` 前先确认是否愿意破这个前提。
- **`assets/` 是内嵌资源的唯一真源**：改 `assets/fonts/unifont.bdf` 就等于改二进制。
- **版本号唯一真源 = `Cargo.toml` 的 `version`**；发版时补 `changelog/v<version>.md`（中文）。

## 注释：`///` 与它修饰的 item 绑定

文档注释是位置绑定的，编译器与 rustfmt 都不校验"这段注释说的到底是不是下面这个东西"。
复制 / 抽取 / 重排时很容易只搬代码、注释留原地，留下一条张冠李戴的注释，照样能编译。

- **搬代码 = 搬注释**：连同上方紧邻的 `///` 一起搬，搬完回头确认原位置没有孤儿注释。
- **语义自检**：注释里提到的形参名 / 返回值 / 字段必须能在紧随其后的签名里找到。`///` 里出现
  "返回""参数"这类函数语义词而下面挂的是 `struct` / `const` / `type`，基本就是贴错了。
- **不留重复文档**：同一段 `///` 出现在两处，至少一处是错的。
- **新增 item 必须有 `///`**：`struct` / `enum`（含每个字段、每个变体）、`const` / `static`
  （成组常量逐个写）、`fn`（`Option` / `Result` 要说明何时 `None`、何时出错）、`type` 别名。
  字段注释写"是什么 + 单位 / 取值 / `None` 的含义"，不要复述字段名。
- 例外的只有测试代码与 `#[cfg(test)]` 块。

## 陷阱

- **测试并行会互踩静态值**：`Shared` 是进程级 `static`，`cargo test` 默认多线程跑同一文件里的
  用例，共享同一份状态。凡是用到共享表的用例，按 `tests/custom.rs` 的做法加
  `static LOCK: Mutex<()>` 串行并在开头清零，别指望测试编号顺序。
- **`examples/` 顶层的每个 `.rs` 都是独立 example target**：辅助模块必须放在子目录里
  （`examples/sim/app/mod.rs` 那样由 `mod app;` 解析），直接在 `examples/` 顶层放没有 `main`
  的文件会编译失败。
- **`icons!` 的路径相对源文件**：`tables.rs` 里写的是 `../assets/icon0.bin`，挪文件时一起改。
- **示例字库是编译期 `include_bytes!`**：给 `examples/sim` 加中文字符串后必须 `make font`
  重新生成 `examples/sim/assets/font/{font,width,cp}.bin`，否则新字符不显示。
  `make run` / `make snapshot` 会自动先跑这一步。
- **改出"没有生效"的编译错误先怀疑 target 缓存**：旧 rlib 会报已经不存在的符号。换目录验证：
  `CARGO_TARGET_DIR=/tmp/kk-check cargo check --all-features`。
- **`app.rs` 的注释声称有两份副本**：`examples/sim/app.rs` 顶部写着与 `ili9341-lcd-kkui/src/app.rs`
  同源（图标各自一份 `assets/`）。那个目录不在本仓内，改动时按需确认。
- **页面身份靠指针比较**：`core::ptr::eq` 判的是 `&'static PageRoute`。同一个页面声明成两个
  `static` 会被当成两个不同页面；`App.routes` 只是无序校验清单，漏列只会静默跳过校验。

## 构建 / 测试

无 `rust-toolchain.toml`、无 CI 配置、无 git hook——格式与检查全靠自觉。

```bash
cargo check                                    # 默认 feature（纯 no_std）
cargo check --all-features
cargo test                                     # 全量测试（部分用例需要 sim / cjk-fonts）
cargo fmt                                      # 提交前跑一次；cargo fmt --check 只核对
cargo clippy --all-features

make                    # 生成示例字库 + 开窗口跑模拟器
make font               # 只生成 examples/sim/assets/font/
make snapshot           # 无头截图（SNAPSHOT=/tmp/sim.png STEPS=40）
make publish-test       # cargo publish --dry-run
```

- `cargo fmt` 不接受 `--bin`，`cargo fmt --check --bin <name>` 会报错，只能用 `cargo fmt --check`。
- 无头跑示例：`cargo run --example sim --features "sim cjk-fonts" -- --snapshot out.png --steps 460`
  （autoplay 自带节奏，`--steps` 决定停在哪一页；460 拍左右是「取值控件」页）。
- 涉及渲染的验证优先断言帧缓冲，不要截图肉看：`tests/frame.rs`、`tests/widgets.rs` 里有现成写法。

## 提交纪律

- 提交信息用中文，按改动性质逐行加前缀：新增 `[+]`、删除 `[-]`、修改 `[*]`；一行一件事，
  行与行之间**不留空行**；一个提交含多类改动就写多行，顺序为 新增 > 删除 > 修改。
- 用**显式路径**暂存（`git add <具体文件>`），不要 `git add -A` / `git add .`；提交前 `git status`
  确认暂存区只含本次改动。
- 纯格式化改动与逻辑改动**分开提交**。
- 不发指令就不要自作主张提交。

## changelog

`changelog/v<version>.md` 给用户看，不是开发日志。类别固定为 新增 / 变更 / 修复 / 移除
（破坏性变更作为「变更」下的子节）。

- **一个功能一条**：同一功能在本版内的后续改动、修复、微调全部并进那条新增条目。
- **一个功能只出现在一个类别下**；被并条目里「破坏性操作 / 默认不启用 / 需要手动迁移 / 已知边界」
  必须进合并后的正文，纯视觉微调与内部重构可以不留。
- **按功能写，不记中间过程**：只写用户能观察到的新能力与边界，不写迭代顺序、不写提交流水。
- 条目的 `src/...` 路径是给维护者看的索引，写文件路径而不是行号。
