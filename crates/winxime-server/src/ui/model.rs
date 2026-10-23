//! 候选栏数据模型：绘制所需的输入数据（候选/字根/布局结果）。

use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use windows_core::HSTRING;

use crate::config::get_colors;
use winxime_ipc::Context;

#[derive(Debug, Clone)]
pub(crate) struct RenderedMetrics {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) hw_width: f32,
    pub(crate) hw_height: f32,
    /// 候选栏自身高度（不含面板区）。
    pub(crate) bar_height: f32,
    pub(crate) item_height: f32,
    pub(crate) item_widths: Vec<f32>,
    pub(crate) selkey_widths: Vec<f32>,
    pub(crate) text_widths: Vec<f32>,
    /// "⋮" 菜单按钮矩形（窗口 DIP 坐标，仅横向布局存在）。
    pub(crate) menu_button: Option<(f32, f32, f32, f32)>,
}

#[derive(Debug, Clone, Default)]
pub struct CandidateModel {
    pub items: Vec<String>,
    pub comments: Vec<String>,
    pub selkeys: Vec<u16>,
    /// Rime 的分页信息：候选栏按 `horizontal`/`cand_per_row` 自己排，没有画
    /// "1/5" 这类页码，所以这两个值目前只写不读。
    #[allow(dead_code)]
    pub total_pages: u32,
    #[allow(dead_code)]
    pub current_page: u32,
    pub font_family: HSTRING,
    pub font_size: f32,
    pub cand_per_row: u32,
    pub horizontal: bool,
    pub use_cursor: bool,
    pub current_sel: usize,
    /// 候选/面板窗口圆角半径（`style.corner_radius`，DIP）。
    ///
    /// 之前绘制里写死 8.0 / 10.0，设置页的「圆角大小」存了也没人读。
    pub corner_radius: f32,
    pub selkey_color: D2D1_COLOR_F,
    pub fg_color: D2D1_COLOR_F,
    pub comment_color: D2D1_COLOR_F,
    pub bg_color: D2D1_COLOR_F,
    pub highlight_fg_color: D2D1_COLOR_F,
    pub highlight_bg_color: D2D1_COLOR_F,
    pub border_color: D2D1_COLOR_F,
}

#[derive(Debug, Clone, Default)]
pub struct RootModel {
    pub letter: char,
    pub root: String,
    pub font_family: HSTRING,
    pub font_size: f32,
    /// 字根浮层圆角（同一份 `style.corner_radius`）。
    pub corner_radius: f32,
    pub primary_color: D2D1_COLOR_F,
    pub bg_color: D2D1_COLOR_F,
    pub fg_color: D2D1_COLOR_F,
}

impl From<(char, String)> for RootModel {
    fn from((letter, root): (char, String)) -> Self {
        let config = crate::config::get();
        let font_family = if config.style.font_family.is_empty() {
            HSTRING::from("Microsoft YaHei UI")
        } else {
            HSTRING::from(config.style.font_family.as_str())
        };
        let (r, g, b) = config.get_primary_color();
        let color_u32 = (r as u32) << 16 | (g as u32) << 8 | b as u32;
        let (_, _, _, selkey_color, _, _, _) = get_colors(color_u32);

        Self {
            letter,
            root,
            font_family,
            font_size: config.style.font_size,
            corner_radius: config.style.corner_radius.clamp(0.0, 32.0),
            primary_color: selkey_color,
            bg_color: D2D1_COLOR_F {
                r: 0.98,
                g: 0.98,
                b: 0.98,
                a: 1.0,
            },
            fg_color: D2D1_COLOR_F {
                r: 0.2,
                g: 0.2,
                b: 0.2,
                a: 1.0,
            },
        }
    }
}

impl From<&Context> for CandidateModel {
    fn from(ctx: &Context) -> Self {
        let config = crate::config::get();
        let font_family = if config.style.font_family.is_empty() {
            HSTRING::from("Microsoft YaHei UI")
        } else {
            HSTRING::from(config.style.font_family.as_str())
        };
        let (r8, g8, b8) = config.get_primary_color();
        let color_u32 = (r8 as u32) << 16 | (g8 as u32) << 8 | b8 as u32;
        let (
            bg_color,
            border_color,
            fg_color,
            selkey_color,
            comment_color,
            highlight_bg_color,
            highlight_fg_color,
        ) = get_colors(color_u32);

        // 分页列数来自用户配置（`style.candidate_count`）。这里就夹住：0 或负数会让
        // 后面的行数计算变成除零（`inf` 一路变成窗口高度的 `inf`），
        // 上限 32 只是防呆（一屏几十列的候选栏没有意义，也没人这么配）。
        let cand_per_row = if config.style.horizontal {
            config.style.candidate_count.clamp(1, 32) as u32
        } else {
            1
        };

        // 候选栏一次显示几个：`style.candidate_count`（文档语义）。
        // 上限 10：选字键只有 1–9 + 0（librime 的 selector 原生支持 0 = 第 10 个），
        // 再多也只能用方向键选，显示出来纯属占地方。
        // rime 一页最多给 `menu/page_size` 个，所以这里再截断一次——
        // page_size 由服务端在部署时对齐（见 server 的 `sync_rime_page_size`）。
        let max_items = config.style.candidate_count.clamp(1, 10) as usize;

        let comments: Vec<String> = ctx
            .candidates
            .comments
            .iter()
            .take(max_items)
            .map(|c| c.str.clone())
            .collect();

        // 选字键：1–9，第 10 个用 0（librime 的 selector 原生按
        // `index = ((ch - XK_0) + 9) % 10` 映射，所以 0 就是第 10 个，不用额外绑定）。
        let selkeys: Vec<u16> = ('1'..='9')
            .map(|c| c as u16)
            .chain(std::iter::once('0' as u16))
            .collect();

        // 候选栏一次显示几个：`style.candidate_count`（文档语义）。
        // rime 一页最多给 `menu/page_size` 个，所以这里再截断一次——
        // page_size 由服务端在部署时对齐（见 server 的 `set_page_size`）。
        let max_items = config.style.candidate_count.clamp(1, 32) as usize;

        Self {
            items: ctx
                .candidates
                .candies
                .iter()
                .take(max_items)
                .map(|c| c.str.clone())
                .collect(),
            comments,
            selkeys,
            total_pages: ctx.candidates.total_pages,
            current_page: ctx.candidates.current_page + 1,
            current_sel: ctx.candidates.highlighted,
            font_family,
            font_size: config.style.font_size,
            corner_radius: config.style.corner_radius.clamp(0.0, 32.0),
            cand_per_row,
            horizontal: config.style.horizontal,
            use_cursor: true,
            selkey_color,
            fg_color,
            comment_color,
            bg_color,
            highlight_fg_color,
            highlight_bg_color,
            border_color,
        }
    }
}
