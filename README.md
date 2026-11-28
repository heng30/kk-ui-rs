# kk-ui-rs

[KK_UI](https://gitee.com/keysking/kk_ui) 的 Rust 移植：单色菜单 UI 运行时。
提供首页图标选择器（静态 XBM 或动图）、可嵌套菜单、只读信息页、应用自绘页、整数/开关编辑器、
分段单选行、位掩码行、枚举选择弹窗、单行输入框（软键盘）、确认框、消息框与 Toast；画布尺寸由应用决定，
布局固定 16px 行高，屏越高菜单能显示的行越多。交互只有上/下/确定三个键：软键盘每行两端
各有一个换行按钮，上下键在行内移动、确定踩按钮换行，光标停在原地可以连按同一个键；新组件的
滑动与滚动用定点弹簧驱动。

```text
应用页面与业务（本 crate 之外）
      ↓
   kk-ui-rs      ← 本 crate：导航、交互、动画、绘制
      ↓
 1 bit 帧缓冲    ← Frame::buffer()
      ↓
显示驱动（由应用实现）
```

## 依赖

只依赖 `core`，没有外部 crate，也没有 `alloc`。可以在主机上直接 `cargo build` 跑
渲染测试（`#![no_std]` 但不用任何外设）。

## 用法

```rust
use kk_ui::{App, Input, KkUi, Status};

// 帧缓冲由应用提供，大小是 `Frame::buffer_len(width, height)`。
const WIDTH: i16 = 128;
const HEIGHT: i16 = 64;
let mut buffer = [0u8; Frame::buffer_len(WIDTH, HEIGHT)];
let mut ui = KkUi::new(&MY_APP, &mut buffer, WIDTH, HEIGHT);
ui.init();

loop {
    let now_ms = /* 单调毫秒 */;
    ui.update(now_ms, Input { keys: 0, encoder_delta: 0 });
    if ui.take_frame_ready() {
        blit(ui.frame()); // 把 `width × height` 的 1 bit 画布送到屏幕
    }
}
```


自绘页实现 `CustomPageHandler`，再作为一条路由挂进 `App.routes`——路由表第 `n` 项就是页面
编号 `n + 1`，页面数据直接挂在路由上，没有第二张需要对齐的表：

```rust
use core::cell::Cell;

static MY_PAGE: MyPage = MyPage::new();

static MY_ROUTE: PageRoute = PageRoute::Custom(&MY_PAGE);
static ROUTES: [&PageRoute; 1] = [&MY_ROUTE];

static MY_APP: App = App {
    root_page: &MY_ROUTE,
    routes: &ROUTES,
    // ...
};

impl CustomPageHandler for MyPage {
    // 回调只拿 `&self`，可变状态放 `Cell`/`Shared` 里。
    fn on_enter(&self, ui: &mut Runtime, page: &'static PageRoute) {}
    fn on_input(&self, ui: &mut Runtime, page: &'static PageRoute, event: InputEvent) {}
    fn on_tick(&self, ui: &mut Runtime, page: &'static PageRoute, now: u32) -> bool { false }
    fn on_draw(&self, ui: &mut Runtime, page: &'static PageRoute, x_offset: i16, clip_x: i16, clip_width: u16) {
        let frame = ui.frame_mut();
        let (width, height) = (frame.width(), frame.height());
        // frame.draw_text / draw_rbox / ...
    }
}
```

## 动图与静态图标

### 动图（`anim-icons`，默认关）

483 个 icons8 动图，每个备 32 / 48 / 64 三档；**每档一个独立静态**，只引用哪一档就只链入
哪一档的字节（一个动图 28 帧）：

| 档位 | 静态名 | 单帧 | 整个动图 |
|---|---|---|---|
| 32×32 | `ACTIVITY_32` | 128 B | 3.5 KiB |
| 48×48 | `ACTIVITY_48` | 288 B | 7.9 KiB |
| 64×64 | `ACTIVITY_64` | 512 B | 14.0 KiB |

```toml
kk-ui-rs = { version = "0.4", features = ["anim-icons"] }
```

```rust
use kk_ui::{anim_icons::ACTIVITY_64, draw_anim_icon, draw_anim_icon_at, Frame};

// 第 12 帧，左上角 (8, 8)；尺寸取自静态自己，不用再传。
draw_anim_icon(&mut frame, 8, 8, &ACTIVITY_64, 12);
// 按经过的毫秒数取帧：24 fps 循环。
draw_anim_icon_at(&mut frame, 8, 8, &ACTIVITY_64, elapsed_ms);
```

`AnimIcon { size: IconSize, frames: &'static [u8] }`：`IconSize` 是 `Px32` / `Px48` /
`Px64`（`px()`、`frame_len()`、`ALL`），每帧 `size × size / 8` 字节，布局与 `Frame` 的
缓冲一致，`frame_count()` 给帧数。未引用的动图不会进固件——实测空程序 435 KiB，多引一个
32 档只多 3.5 KiB。

自己的动图放 `<你的 crate>/assets/icons/animators/<文件名>.{32,48,64}.bin`，用宏声明
（路径靠 `CARGO_MANIFEST_DIR` 拼，不写相对路径）：

```rust
kk_ui::anim_icons! {
    /// 32×32 的转圈。
    SPINNER_32 = "spinner" @ Px32,
}
```

首页入口的图标同样可以是动图（同样要开 `anim-icons`），组件会按 24 fps 换帧重画整页；
只有静态图标的首页不重画：

```rust
static HOME_ITEMS: [HomeItem; 1] = [HomeItem {
    label: "动图",
    icon: HomeIcon::Anim(&ACTIVITY_32), // 静态图标写 HomeIcon::Static(XBM)
    target: &ROUTE_ANIMS,
}];
```

### 静态图标

库自带四张 32×32 XBM（`assets/icons/statics/`）：`ICON_SETTINGS`、`ICON_MONITOR`、
`ICON_WAVE`、`ICON_ABOUT`，用 `Frame::draw_xbm(x, y, 32, 32, ICON_SETTINGS)` 画，或者直接
塞进 `HomeIcon::Static(ICON_SETTINGS)`。自己的图标同理：

```rust
kk_ui::static_icons! {
    /// 32×32 的齿轮。
    ICON_GEAR = "gear",
}
```

`.bin` 放 `<你的 crate>/assets/icons/statics/gear.bin`，用 `kkui-tool icon` 从图片生成。

## 字体：全局缓存 + 内嵌点阵（`cjk-fonts`）

整个 UI 只有一个当前字体，放在库内全局缓存里，页面与弹窗不再各自持有：
`Frame::draw_text` / `Frame::text_width` 直接读它。设置函数在程序初始化时调用一次：

```rust
// 启用 cjk-fonts 特性时默认就是内嵌字库，初始化无需任何调用。
// 要换成自己的 16×16 字库：
kk_ui::set_font(&MY_GLYPHS);
// 要换回内嵌字库：
kk_ui::set_font(&kk_ui::cjk::GLYPHS);
```

字库是「点阵 / 宽度 / 码点」三张平行静态表，自己的表只需按同一布局提供：

```rust
pub struct Glyphs {
    pub bitmaps: &'static [u8],     // 每个字形 32 字节，16 行 × 2 字节，行内 MSB 在左
    pub widths: &'static [u8],      // 像素宽度，全角 16、半角按实际
    pub codepoints: &'static [u8],  // u16 小端升序，二分查找
}
```

不开 `cjk-fonts` 时初始未设置字体，文字绘制为空操作，须先 `set_font`。

`cjk-fonts` 的数据由 `build.rs` 在编译期从仓库自带的 `assets/fonts/unifont.bdf`
里抠出，写进 `OUT_DIR` 后嵌进库：GB2312 常用汉字、中文标点、全角字符与制表符，
以及全部 ASCII 大小写字母和常用英文符号，所以中英混排文本（`FPS: 80%`、
`返回Back`）可以直接画：

```bash
cargo build --features cjk-fonts
```

```rust
let mut frame = Frame::new(&mut buffer, 128, 64);
// Frame 的文字接口走全局字体，也可以直接查内嵌字库。
frame.draw_text(0, 0, "屏幕亮度：80% FPS: 30");

if let Some(bitmap) = kk_ui::cjk::GLYPHS.glyph('中') {
    // 16×16 点阵，行内 MSB 在左，每行 2 字节
}
```

字形固定 16×16：全角 16 px 步进，半角（ASCII 及全角区里的窄字）按 BDF 原始宽度。
码点二分查找，字库约 7170 个字形（95 个 ASCII + 6763 个 GB2312 汉字 + 中文标点、
全角字符、制表符），编进库约 245 KB。`assets/fonts/unifont.bdf` 是仓库内副本
（9 MB），没装 Unifont 的机器也能构建；不开 `cjk-fonts` 时这部分完全不参与编译。

## 资源工具 `kkui-tool`

包内唯一的可执行程序（`bin/kkui-tool.rs`），它依赖的 `clap` / `image` 归 `tool` 特性，
所以安装要带上特性：`cargo install --path . --features tool`（本地跑用
`cargo run --bin kkui-tool --features tool --`）。

```text
kkui-tool font <目录或文件>... [--out font]  # 采集字符，产出 3 张字库表 + text.txt
kkui-tool icon <图片> [--size 32] [-o 输出]   # png/jpeg/webp → XBM 位图（raw .bin）
```

`font` 的字符来源分两部分：**可打印 ASCII（英文字母、数字、英文标点）始终收录**；中文
则靠扫描——目录只认 `.rs` 文件（跳过 `target`/`.git`/`node_modules`），且只取字符串与
字符字面量里的字，**注释（含 `///` 文档注释）不算**。显式传入的非 `.rs` 文件按纯文本收全部
字符，方便把上一次生成的 `text.txt` 再喂回来。

字符按码点升序从 `assets/fonts/unifont.bdf` 里抠出字模，产出与 `build.rs` 同构的三张平行表，
`include_bytes!` 后即可组成 `Glyphs` 交给 `set_font`；同目录的 `text.txt` 是采集到的字符清单。
BDF 在编译期就 `include_bytes!` 进了工具（装出来的二进制约 21 MB），所以不需要传字库路径，
也不依赖 `cjk-fonts` 特性；能取到哪些码点由 BDF 决定，取不到的字符会被列出来但不入表。

`icon` 把图缩到 `--size` 见方再二值化（深色算点亮、透明算熄灭），按 `Frame::draw_xbm`
的位序写 raw 二进制：低位在前、每行 8 像素对齐，32×32 正好 128 字节。

本目录的 `Makefile` 把这两步串起来了：`make` 先给 `examples/sim` 生成字库再跑模拟器，
`make font` 只生成字库，`make snapshot` 出无头截图。

## 本地模拟器（宿主机）

`sim` 特性编译出一个小框架（`kk_ui::sim`，源码在 `src/sim.rs`），把 1 bit 画布显示到
eframe 窗口，并把窗口按键注入应用。库本体仍是 `#![no_std]`，不开这个特性时
`eframe` / `png` 都不会被拉进来。

```rust
kk_ui::sim::run(
    kk_ui::sim::Options { title: "demo", width: 160, height: 120, scale: 2 },
    move |ctx| {
        // 每帧调用一次：ctx.now 是毫秒时间戳，ctx.keys 是本帧按住的键
        ui.update(ctx.now, Input { keys: ctx.keys, encoder_delta: 0 });
        ctx.show(ui.frame()); // 交出这帧要显示的画布
    },
)?;
```

窗口固定大小、画布与按键栏都居中，底部一排大号按钮（Up/Down/OK/Snap）与键盘等价：
`↑`/`←`/`k`/`h` 上、`↓`/`→`/`j`/`l` 下、回车确定、`S` 存一张 `sim-<n>.png`。
截图是 8 位灰度 PNG，`kk_ui::sim::save_png(frame, scale, path)` 也可以直接调，
无头调试时不必开窗口。

`examples/sim/` 是完整例子。无头模式 `cargo run --example sim --features "sim cjk-fonts" -- --snapshot out.png --steps 40`
让 Demo 自己走 40 拍再落盘。

## 许可证

跟随上游 KK_UI，MIT。静态图标来自上游标准 Demo；动图由 icons8.com 的动画图标栅格化而来，
免费使用时需在项目里署名 icons8.com。
