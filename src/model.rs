//! 应用描述表：路由、页面、绑定与文案。
//!
//! 与 C 版一致，全部是编译期常量表，UI 运行时只读；应用的可变状态通过
//! [`Shared`] 与 [`Text`] 这类共享单元暴露给表项。

use crate::ui::CustomPageHandler;
use core::cell::Cell;

/// 事件编号，`0` 表示无事件。
pub type EventId = u16;
/// 一页自绘页：路由持有它，回调由它接收。
///
/// 页面状态靠实现内部的 `Cell`/`Shared` 自行维护，所以要求 `Sync` 才能放进常量表。
pub type CustomPage = &'static (dyn CustomPageHandler + Sync);

/// 无事件。
pub const EVENT_NONE: EventId = 0;

/// 按键位：上。
pub const KEY_UP: u8 = 1 << 0;
/// 按键位：下。
pub const KEY_DOWN: u8 = 1 << 1;
/// 按键位：确定。
pub const KEY_OK: u8 = 1 << 2;

/// 运行时文本缓冲的最大字节数，按 UTF-8 计；32 字节能放下 10 个汉字。
pub const TEXT_LEN: usize = 32;

/// 文本编辑器缓冲的最大字节数，按 UTF-8 计；64 字节能放下 21 个汉字。
pub const LINE_INPUT_LEN: usize = 64;

/// 单核裸机下的共享可变值：只在 UI 主循环与其回调里访问。
pub struct Shared<T>(Cell<T>);

// SAFETY: 只在单核上由 UI 主循环顺序访问，不跨核、不在中断里使用。
unsafe impl<T> Sync for Shared<T> {}

impl<T: Copy> Shared<T> {
    /// 创建初值。
    pub const fn new(value: T) -> Self {
        Self(Cell::new(value))
    }

    /// 读取当前值。
    pub fn get(&self) -> T {
        self.0.get()
    }

    /// 写入新值。
    pub fn set(&self, value: T) {
        self.0.set(value)
    }
}

/// 长度受限的共享文本，供会变化的表项（如运行时间、亮度）使用。
pub struct Text {
    /// 内容字节，未用部分补 0。
    bytes: Cell<[u8; TEXT_LEN]>,
    /// 有效字节数，按 UTF-8 计。
    len: Cell<u8>,
}

// SAFETY: 与 [`Shared`] 相同，只在单核 UI 主循环里顺序访问。
unsafe impl Sync for Text {}

impl Default for Text {
    /// 空文本。
    fn default() -> Self {
        Self::new()
    }
}

impl Text {
    /// 空文本。
    pub const fn new() -> Self {
        Self {
            bytes: Cell::new([0; TEXT_LEN]),
            len: Cell::new(0),
        }
    }

    /// 由字符串字面量构造；超出 [`TEXT_LEN`] 的部分按字符边界截断。
    pub const fn literal(text: &str) -> Self {
        let source = text.as_bytes();
        let len = fit(source);
        let mut bytes = [0u8; TEXT_LEN];
        let mut index = 0;
        while index < len {
            bytes[index] = source[index];
            index += 1;
        }
        Self {
            bytes: Cell::new(bytes),
            len: Cell::new(len as u8),
        }
    }

    /// 覆盖内容；超出 [`TEXT_LEN`] 的部分按字符边界截断。
    pub fn set(&self, text: &str) {
        let source = text.as_bytes();
        let mut bytes = [0u8; TEXT_LEN];
        let len = fit(source);
        bytes[..len].copy_from_slice(&source[..len]);
        self.bytes.set(bytes);
        self.len.set(len as u8);
    }

    /// 在栈上复制一份内容后交给闭包，返回值原样透传。
    pub fn with<R>(&self, f: impl FnOnce(&str) -> R) -> R {
        let bytes = self.bytes.get();
        let len = self.len.get() as usize;
        f(core::str::from_utf8(&bytes[..len]).unwrap_or(""))
    }
}

/// 不超过 `cap` 且落在字符边界上的字节数。
const fn fit_in(source: &[u8], cap: usize) -> usize {
    let mut len = if source.len() < cap {
        source.len()
    } else {
        cap
    };
    while len > 0 && len < source.len() && source[len] & 0xC0 == 0x80 {
        len -= 1;
    }
    len
}

/// 不超过 [`TEXT_LEN`] 且落在字符边界上的字节数。
const fn fit(source: &[u8]) -> usize {
    fit_in(source, TEXT_LEN)
}

/// 可编辑的共享文本，供 [`LineInputBinding`] 与文本编辑器使用。
///
/// 与 [`Text`] 的访问约定相同，区别是容量更大且带末尾插入、删除。
pub struct LineInput {
    /// 内容字节，未用部分补 0。
    bytes: Cell<[u8; LINE_INPUT_LEN]>,
    /// 有效字节数，按 UTF-8 计。
    len: Cell<u8>,
}

// SAFETY: 与 [`Text`] 相同，只在单核 UI 主循环里顺序访问。
unsafe impl Sync for LineInput {}

impl Default for LineInput {
    /// 空文本。
    fn default() -> Self {
        Self::new()
    }
}

impl LineInput {
    /// 空文本。
    pub const fn new() -> Self {
        Self {
            bytes: Cell::new([0; LINE_INPUT_LEN]),
            len: Cell::new(0),
        }
    }

    /// 由字符串字面量构造；超出 [`LINE_INPUT_LEN`] 的部分按字符边界截断。
    pub const fn literal(text: &str) -> Self {
        let source = text.as_bytes();
        let len = fit_in(source, LINE_INPUT_LEN);
        let mut bytes = [0u8; LINE_INPUT_LEN];
        let mut index = 0;
        while index < len {
            bytes[index] = source[index];
            index += 1;
        }
        Self {
            bytes: Cell::new(bytes),
            len: Cell::new(len as u8),
        }
    }

    /// 覆盖内容；超出 [`LINE_INPUT_LEN`] 的部分按字符边界截断。
    pub fn set(&self, text: &str) {
        let source = text.as_bytes();
        let mut bytes = [0u8; LINE_INPUT_LEN];
        let len = fit_in(source, LINE_INPUT_LEN);
        bytes[..len].copy_from_slice(&source[..len]);
        self.bytes.set(bytes);
        self.len.set(len as u8);
    }

    /// 在栈上复制一份内容后交给闭包，返回值原样透传。
    pub fn with<R>(&self, f: impl FnOnce(&str) -> R) -> R {
        let bytes = self.bytes.get();
        let len = self.len.get() as usize;
        f(core::str::from_utf8(&bytes[..len]).unwrap_or(""))
    }

    /// 在末尾追加一个字符；放不下时整字丢弃，已存内容不受影响。
    pub fn push(&self, c: char) {
        let mut encoded = [0u8; 4];
        let encoded = c.encode_utf8(&mut encoded).as_bytes();
        let len = self.len.get() as usize;
        if len + encoded.len() > LINE_INPUT_LEN {
            return;
        }
        let mut bytes = self.bytes.get();
        bytes[len..len + encoded.len()].copy_from_slice(encoded);
        self.bytes.set(bytes);
        self.len.set((len + encoded.len()) as u8);
    }

    /// 删除末尾一个完整字符；空文本无操作。
    pub fn pop(&self) {
        let len = self.len.get() as usize;
        if len == 0 {
            return;
        }
        let mut bytes = self.bytes.get();
        let mut end = len - 1;
        while end > 0 && bytes[end] & 0xC0 == 0x80 {
            end -= 1;
        }
        for byte in &mut bytes[end..len] {
            *byte = 0;
        }
        self.bytes.set(bytes);
        self.len.set(end as u8);
    }
}

/// 一次刷新传入的原始输入。
#[derive(Clone, Copy, Default)]
pub struct Input {
    /// 按键位掩码，取 [`KEY_UP`] / [`KEY_DOWN`] / [`KEY_OK`] 的组合。
    pub keys: u8,
    /// 编码器增量，正值为向下、负值为向上。
    pub encoder_delta: i16,
}

/// 归一化后的输入动作。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    /// 向上。
    Up,
    /// 向下。
    Down,
    /// 确定。
    Ok,
}

/// 输入来源。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InputSource {
    /// 首次按下。
    Press,
    /// 长按重复。
    Repeat,
    /// 编码器。
    Encoder,
}

/// 分发给页面或弹窗的输入事件。
#[derive(Clone, Copy)]
pub struct InputEvent {
    /// 动作。
    pub action: InputAction,
    /// 来源。
    pub source: InputSource,
    /// 一次性合并的步数。
    pub steps: u16,
}

/// 页面类型。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PageType {
    /// 首页图标选择器。
    Home,
    /// 菜单。
    Menu,
    /// 只读信息页。
    Info,
    /// 应用自绘页。
    Custom,
}

/// 一条路由：页面描述本身，同时充当页面句柄——`&'static PageRoute` 就是页面身份。
///
/// 每个页面只需一个 `static`，引用它的地方一律用 `&ROUTE_XXX`，没有编号也没有下标。
pub enum PageRoute {
    /// 首页图标选择器。
    Home(&'static HomePage),
    /// 菜单。
    Menu(&'static MenuPage),
    /// 只读信息页。
    Info(&'static InfoPage),
    /// 应用自绘页。
    Custom(CustomPage),
}

impl PageRoute {
    /// 页面类型标签。
    pub fn kind(&self) -> PageType {
        match self {
            Self::Home(_) => PageType::Home,
            Self::Menu(_) => PageType::Menu,
            Self::Info(_) => PageType::Info,
            Self::Custom(_) => PageType::Custom,
        }
    }
}

/// 首页的一个图标入口。
pub struct HomeItem {
    /// 文字标签。
    pub label: &'static str,
    /// 32×32 XBM 图标。
    pub icon: &'static [u8],
    /// 选中后进入的页面。
    pub target: &'static PageRoute,
}

/// 一组首页入口。
pub struct HomePage {
    /// 入口表。
    pub items: &'static [HomeItem],
}

/// 菜单项被激活后做什么。
#[derive(Clone, Copy)]
pub enum MenuAction {
    /// 进入子页面。
    Page(&'static PageRoute),
    /// 触发事件。
    Event(EventId),
    /// 整数编辑器。
    Int(&'static IntBinding),
    /// 开关编辑器。
    Bool(&'static BoolBinding),
    /// 分段单选行，按 OK 循环换档。
    Chip(&'static ChipBinding),
    /// 位掩码行，按 OK 翻转游标位并前进。
    Bits(&'static BitsBinding),
    /// 枚举选择弹窗。
    Choice(&'static ChoiceBinding),
    /// 文本编辑器。
    LineInput(&'static LineInputBinding),
    /// 确认框。
    Confirm(&'static ConfirmDesc),
}

/// 菜单项。
pub struct MenuItem {
    /// 标签。
    pub label: &'static str,
    /// 激活后的动作。
    pub action: MenuAction,
}

/// 菜单页。
pub struct MenuPage {
    /// 标题。
    pub title: &'static str,
    /// 条目表。
    pub items: &'static [MenuItem],
    /// 可选的条目状态表，每字节压 4 个 2 bit 状态。
    pub states: Option<&'static [Shared<u8>]>,
}

/// 条目状态。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ItemState {
    /// 正常。
    Normal = 0,
    /// 隐藏。
    Hidden = 1,
    /// 显示但不可选。
    Disabled = 2,
    /// 保留值。
    Reserved = 3,
}

/// 信息页的一行。
pub struct InfoRow {
    /// 名称。
    pub name: &'static str,
    /// 取值；空文本表示不绘制右列。
    pub value: &'static Text,
}

/// 信息页。
pub struct InfoPage {
    /// 标题；空表示不画标题栏。
    pub title: &'static str,
    /// 行表。
    pub rows: &'static [InfoRow],
}

/// 整数编辑器绑定。
pub struct IntBinding {
    /// 标题。
    pub title: &'static str,
    /// 绑定的值。
    pub value: &'static Shared<i32>,
    /// 最小值。
    pub minimum: i32,
    /// 最大值。
    pub maximum: i32,
    /// 单步增量。
    pub step: u32,
    /// 单位后缀。
    pub unit: &'static str,
    /// 提交后入队的事件。
    pub changed_event: EventId,
}

/// 开关编辑器绑定。
pub struct BoolBinding {
    /// 标题。
    pub title: &'static str,
    /// 绑定的值。
    pub value: &'static Shared<bool>,
    /// 提交后入队的事件。
    pub changed_event: EventId,
}

/// 分段单选绑定：值就是选中的段号。
///
/// 画在菜单行的右列，按 OK 在段之间循环，选中即生效。
pub struct ChipBinding {
    /// 各段的文字。
    pub levels: &'static [&'static str],
    /// 绑定的值。
    pub value: &'static Shared<u8>,
    /// 换档后入队的事件。
    pub changed_event: EventId,
}

/// 位掩码绑定：一位一个方片，按 OK 打开弹窗逐位开关。
///
/// 弹窗里上/下选位、确定翻转，走到列表尽头按确定会把焦点交给按钮；提交后入队事件。
pub struct BitsBinding {
    /// 弹窗标题。
    pub title: &'static str,
    /// 每位的名字，位数就是它的长度，最多 [`MAX_BITS`]。
    pub labels: &'static [&'static str],
    /// 绑定的值，第 `n` 位对应第 `n` 个名字。
    pub value: &'static Shared<u32>,
    /// 提交后入队的事件。
    pub changed_event: EventId,
}

/// 文本绑定：菜单行右列显示当前文字，按 OK 打开文本编辑器改写。
pub struct LineInputBinding {
    /// 标题。
    pub title: &'static str,
    /// 绑定的文字。
    pub value: &'static LineInput,
    /// 提交后入队的事件。
    pub changed_event: EventId,
}

/// 枚举选择绑定：值就是标签表里的下标。
pub struct ChoiceBinding {
    /// 标题。
    pub title: &'static str,
    /// 可选标签。
    pub labels: &'static [&'static str],
    /// 绑定的值。
    pub value: &'static Shared<u16>,
    /// 提交后入队的事件。
    pub changed_event: EventId,
}

/// 一行最多放得下的方片数。
pub const MAX_BITS: u8 = 8;

/// 确认框描述。
pub struct ConfirmDesc {
    /// 正文，可用 `\n` 分成两行。
    pub text: &'static str,
    /// 确认事件。
    pub confirmed_event: EventId,
    /// 取消事件。
    pub cancelled_event: EventId,
}

/// 固定文案。
pub struct Texts {
    /// 菜单里的返回项。
    pub return_text: &'static str,
    /// 取消。
    pub cancel_text: &'static str,
    /// 确定。
    pub confirm_text: &'static str,
    /// 开。
    pub on_text: &'static str,
    /// 关。
    pub off_text: &'static str,
    /// 消息框标题。
    pub message_title: &'static str,
}

/// 整个应用描述。
pub struct App {
    /// 启动页。
    pub root_page: &'static PageRoute,
    /// 页面清单；顺序无关，只在初始化时用于校验。
    pub routes: &'static [&'static PageRoute],
    /// 整数绑定清单；顺序无关，只在初始化时用于校验。
    pub int_bindings: &'static [&'static IntBinding],
    /// 开关绑定清单；顺序无关，只在初始化时用于校验。
    pub bool_bindings: &'static [&'static BoolBinding],
    /// 文本绑定清单；顺序无关，只在初始化时用于校验。
    pub line_inputs: &'static [&'static LineInputBinding],
    /// 确认框清单；顺序无关，只在初始化时用于校验。
    pub confirm_descs: &'static [&'static ConfirmDesc],
    /// 文案。
    pub texts: Texts,
}
