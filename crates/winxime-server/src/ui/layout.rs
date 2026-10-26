//! 候选栏/字根提示的布局测量：用 DirectWrite 量取文本尺寸，产出 RenderedMetrics。

use windows::Win32::Graphics::DirectWrite::{
    IDWriteFactory1, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
    DWRITE_FONT_WEIGHT_NORMAL, DWRITE_TEXT_METRICS,
};
use windows_core::{w, HSTRING};

use super::model::{CandidateModel, RenderedMetrics, RootModel};
use super::panel::{panel_extra_height, PanelPage, MENU_BUTTON_GAP, MENU_BUTTON_SIZE, MENU_STRIP_HEIGHT, PANEL_MIN_WIDTH};
use super::{BLUR_RADIUS, COL_SPACING, MARGIN, MIN_WIDTH, ROW_SPACING};

/// 候选区相对窗口左侧的偏移：横向布局左侧要留 ⋮ 按钮。
fn candidate_offset(horizontal: bool) -> f32 {
    if horizontal {
        MENU_BUTTON_SIZE + MENU_BUTTON_GAP
    } else {
        0.0
    }
}

/// 候选栏行数。
///
/// `cand_per_row` 是**用户配置**来的（`style.candidate_count`），可能是 0 或负数。
/// 直接拿它当除数会得到 `inf`，一路传成窗口高度的 `inf`（`SetWindowPos` 收到无穷大）。
/// 这里统一按「至少 1 列」算，调用方不必再各自防。
fn row_count(items: usize, cand_per_row: usize) -> f32 {
    let columns = cand_per_row.max(1) as f32;
    (items as f32 / columns).ceil().max(1.0)
}

    pub(crate) fn calculate_client_rect(
        dwrite: &IDWriteFactory1,
        model: &CandidateModel,
        dpi: f32,
        panel_page: Option<PanelPage>,
    ) -> Result<RenderedMetrics, String> {
        unsafe {
            let scale = dpi / 96.0;

            let text_format = dwrite
                .CreateTextFormat(
                    &model.font_family,
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    model.font_size,
                    w!("zh-CN"),
                )
                .map_err(|e| format!("CreateTextFormat failed: {:?}", e))?;

            let mut item_height = 0.0f32;
            let mut item_widths = Vec::new();
            let mut selkey_widths = Vec::new();
            let mut text_widths = Vec::new();
            let padding_x = 6.0;
            let padding_y = 4.0;

            for (i, item) in model.items.iter().enumerate() {
                // 0 = 不画选字键（内置计算器的候选 label 为空：数字是算式输入），
                // 宽度按 0 算，文本直接从候选左边开始。
                let selkey = model.selkeys.get(i).copied().unwrap_or(0);

                let mut item_metrics = DWRITE_TEXT_METRICS::default();
                let mut comment_metrics = DWRITE_TEXT_METRICS::default();

                let mut selkey_width = 0.0f32;
                if selkey != 0 {
                    let selkey_buf = [selkey, '.' as u16];
                    let mut selkey_metrics = DWRITE_TEXT_METRICS::default();
                    dwrite
                        .CreateTextLayout(&selkey_buf, &text_format, f32::MAX, f32::MAX)
                        .map_err(|e| format!("CreateTextLayout for selkey failed: {:?}", e))?
                        .GetMetrics(&mut selkey_metrics)
                        .map_err(|e| format!("GetMetrics for selkey failed: {:?}", e))?;
                    selkey_width = selkey_metrics.widthIncludingTrailingWhitespace;
                    item_height = item_height.max(selkey_metrics.height + 2.0 * padding_y);
                }

                let item_hstring = HSTRING::from(item);
                dwrite
                    .CreateTextLayout(&item_hstring, &text_format, f32::MAX, f32::MAX)
                    .map_err(|e| format!("CreateTextLayout for item failed: {:?}", e))?
                    .GetMetrics(&mut item_metrics)
                    .map_err(|e| format!("GetMetrics for item failed: {:?}", e))?;

                let comment = model.comments.get(i).cloned().unwrap_or_default();
                let comment_hstring = HSTRING::from(&comment);
                dwrite
                    .CreateTextLayout(&comment_hstring, &text_format, f32::MAX, f32::MAX)
                    .map_err(|e| format!("CreateTextLayout for comment failed: {:?}", e))?
                    .GetMetrics(&mut comment_metrics)
                    .map_err(|e| format!("GetMetrics for comment failed: {:?}", e))?;

                let text_width = item_metrics.widthIncludingTrailingWhitespace;
                let comment_width = if comment.is_empty() {
                    0.0
                } else {
                    comment_metrics.widthIncludingTrailingWhitespace + 4.0
                };
                selkey_widths.push(selkey_width);
                text_widths.push(text_width);

                let item_width = selkey_width + text_width + comment_width + 2.0 * padding_x;
                item_widths.push(item_width);
                item_height = item_height.max(item_metrics.height + 2.0 * padding_y);
            }

            let items_len = model.items.len() as f32;
            if items_len == 0.0 {
                return Ok(RenderedMetrics {
                    width: 100.0,
                    height: 30.0,
                    hw_width: ((100.0 + BLUR_RADIUS * 2.0) * scale).ceil(),
                    hw_height: ((30.0 + BLUR_RADIUS * 2.0) * scale).ceil(),
                    bar_height: 30.0,
                    item_height: 20.0,
                    item_widths: Vec::new(),
                    selkey_widths: Vec::new(),
                    text_widths: Vec::new(),
                    menu_button: None,
                });
            }

            let cand_per_row = model.cand_per_row as usize;
            let cpr = cand_per_row.max(1);
            let rows = row_count(model.items.len(), cand_per_row);
            let strip = if model.horizontal {
                0.0
            } else {
                MENU_STRIP_HEIGHT
            };
            let bar_height = strip + rows * item_height + (rows - 1.0) * ROW_SPACING + 2.0 * MARGIN;

            let offset = candidate_offset(model.horizontal);
            let mut max_row_width = MIN_WIDTH;
            for row_start in (0..model.items.len()).step_by(cpr) {
                let row_end = std::cmp::min(row_start + cpr, model.items.len());
                let row_width: f32 = item_widths[row_start..row_end].iter().copied().sum::<f32>()
                    + (row_end - row_start - 1) as f32 * COL_SPACING
                    + 2.0 * MARGIN
                    + offset;
                max_row_width = max_row_width.max(row_width);
            }
            // 横向：左侧 ⋮ 已算进 offset；纵向：顶部条带里居中放 ⋯，不再占右侧宽度。
            let mut width = if model.horizontal {
                max_row_width
            } else {
                max_row_width.max(MENU_BUTTON_SIZE + 2.0 * MARGIN)
            };

            // 面板横向/纵向都允许展开。
            let panel_height = panel_page.map(panel_extra_height);
            if panel_height.is_some() {
                width = width.max(PANEL_MIN_WIDTH);
            }
            let height = bar_height + panel_height.unwrap_or(0.0);

            let menu_button = if model.horizontal {
                Some((
                    MARGIN,
                    MARGIN,
                    MARGIN + MENU_BUTTON_SIZE,
                    MARGIN + item_height,
                ))
            } else {
                Some((
                    (width - MENU_BUTTON_SIZE) / 2.0,
                    MARGIN,
                    (width + MENU_BUTTON_SIZE) / 2.0,
                    MARGIN + strip,
                ))
            };

            let hw_width = ((width + BLUR_RADIUS * 2.0) * scale).ceil();
            let hw_height = ((height + BLUR_RADIUS * 2.0) * scale).ceil();

            Ok(RenderedMetrics {
                width,
                height,
                hw_width,
                hw_height,
                bar_height,
                item_height,
                item_widths,
                selkey_widths,
                text_widths,
                menu_button,
            })
        }
    }

    /// 候选命中测试：窗口客户区 DIP 坐标 → 候选下标。
    ///
    /// 必须与 `paint.rs` 的摆放递推**同源**（首项 `MARGIN`，逐项加宽度 +
    /// `COL_SPACING`，满 `cand_per_row` 换行并加 `ROW_SPACING`，窗口坐标减
    /// `BLUR_RADIUS`）——「画得出来却点不到」是这类自绘 UI 最典型的 bug。
    pub(crate) fn candidate_item_at(
        model: &CandidateModel,
        metrics: &RenderedMetrics,
        x: f32,
        y: f32,
    ) -> Option<usize> {
        if model.items.is_empty() {
            return None;
        }
        let cx = x - BLUR_RADIUS;
        let cy = y - BLUR_RADIUS;
        if cx < 0.0 || cy < 0.0 || cy >= metrics.bar_height {
            return None;
        }
        let columns = (model.cand_per_row as usize).max(1);
        let start_x = MARGIN + candidate_offset(model.horizontal);
        let start_y = MARGIN
            + if model.horizontal {
                0.0
            } else {
                MENU_STRIP_HEIGHT
            };
        let mut item_x = start_x;
        let mut item_y = start_y;
        for (i, _) in model.items.iter().enumerate() {
            let width = metrics.item_widths.get(i).copied().unwrap_or(0.0);
            if cx >= item_x && cx < item_x + width && cy >= item_y && cy < item_y + metrics.item_height
            {
                return Some(i);
            }
            if (i + 1) % columns == 0 {
                item_x = start_x;
                item_y += metrics.item_height + ROW_SPACING;
            } else {
                item_x += width + COL_SPACING;
            }
        }
        None
    }

    pub(crate) fn calculate_root_rect(
        dwrite: &IDWriteFactory1,
        model: &RootModel,
        dpi: f32,
    ) -> Result<RenderedMetrics, String> {
        unsafe {
            let scale = dpi / 96.0;

            let text_format = dwrite
                .CreateTextFormat(
                    &model.font_family,
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    model.font_size,
                    w!("zh-CN"),
                )
                .map_err(|e| format!("CreateTextFormat failed: {:?}", e))?;

            let letter_buf = [model.letter as u16];

            let mut letter_metrics = DWRITE_TEXT_METRICS::default();
            dwrite
                .CreateTextLayout(&letter_buf, &text_format, f32::MAX, f32::MAX)
                .map_err(|e| format!("CreateTextLayout for letter failed: {:?}", e))?
                .GetMetrics(&mut letter_metrics)
                .map_err(|e| format!("GetMetrics for letter failed: {:?}", e))?;

            let root_hstring = HSTRING::from(&model.root);
            let mut root_metrics = DWRITE_TEXT_METRICS::default();
            dwrite
                .CreateTextLayout(&root_hstring, &text_format, f32::MAX, f32::MAX)
                .map_err(|e| format!("CreateTextLayout for root failed: {:?}", e))?
                .GetMetrics(&mut root_metrics)
                .map_err(|e| format!("GetMetrics for root failed: {:?}", e))?;

            let letter_width = letter_metrics.widthIncludingTrailingWhitespace;
            let root_width = root_metrics.widthIncludingTrailingWhitespace;
            let text_height = model.font_size;

            let key_bg_width = letter_width + 16.0;
            let key_bg_height = 24.0;
            let padding = 12.0;

            let width = (padding + key_bg_width + 8.0 + root_width + padding).max(80.0);
            let height = (key_bg_height + padding).max(36.0);

            let hw_width = ((width + BLUR_RADIUS * 2.0) * scale).ceil();
            let hw_height = ((height + BLUR_RADIUS * 2.0) * scale).ceil();

            Ok(RenderedMetrics {
                width,
                height,
                hw_width,
                hw_height,
                bar_height: height,
                item_height: text_height,
                item_widths: vec![root_width],
                selkey_widths: vec![letter_width],
                text_widths: vec![root_width],
                menu_button: None,
            })
        }
    }


#[cfg(test)]
mod tests {
    use super::*;

    /// 行数必须永远有限：`cand_per_row` 是用户配置来的，0 会让除法变成 `inf`，
    /// 而 `bar_height` / `hw_height` 会一路跟着变成 `inf`（窗口尺寸就废了）。
    #[test]
    fn row_count_never_divides_by_zero() {
        // 0 列 = 按 1 列处理，不是崩溃、也不是 inf。
        assert_eq!(row_count(5, 0), 5.0);
        assert_eq!(row_count(1, 0), 1.0);
        assert!(row_count(5, 0).is_finite());
    }

    /// 常规分页：正好整除、有余数、单列。
    #[test]
    fn row_count_matches_page_size() {
        assert_eq!(row_count(5, 5), 1.0);
        assert_eq!(row_count(6, 5), 2.0);
        assert_eq!(row_count(10, 5), 2.0);
        assert_eq!(row_count(5, 1), 5.0);
        assert_eq!(row_count(9, 3), 3.0);
    }

    /// 极端值不能溢出成 inf / NaN（`usize::MAX` 列只是「一行全放下」）。
    #[test]
    fn row_count_is_finite_for_extreme_inputs() {
        assert!(row_count(usize::MAX, usize::MAX).is_finite());
        assert!(row_count(usize::MAX, 1).is_finite());
        assert_eq!(row_count(3, usize::MAX), 1.0);
    }

    fn hit_model() -> CandidateModel {
        CandidateModel {
            items: vec!["a".into(), "b".into(), "c".into()],
            cand_per_row: 2,
            ..CandidateModel::default()
        }
    }

    fn hit_metrics() -> RenderedMetrics {
        RenderedMetrics {
            width: 200.0,
            height: 80.0,
            hw_width: 200.0,
            hw_height: 80.0,
            bar_height: 80.0,
            item_height: 20.0,
            item_widths: vec![60.0, 70.0, 80.0],
            selkey_widths: vec![0.0, 0.0, 0.0],
            text_widths: vec![40.0, 50.0, 60.0],
            menu_button: None,
        }
    }

    /// 两列三行：第 1 项、第 2 项、换行后的第 3 项都能命中。
    #[test]
    fn candidate_hit_matches_paint_layout() {
        let model = hit_model();
        let metrics = hit_metrics();
        // 窗口坐标 = 内容坐标 + BLUR_RADIUS；纵向布局顶部有 ⋯ 条带。
        let top = MARGIN + MENU_STRIP_HEIGHT;
        let at = |x: f32, y: f32| candidate_item_at(&model, &metrics, x + BLUR_RADIUS, y + BLUR_RADIUS);
        assert_eq!(at(MARGIN + 5.0, top + 5.0), Some(0));
        assert_eq!(at(MARGIN + 60.0 + COL_SPACING + 5.0, top + 5.0), Some(1));
        // 第二行：y = top + item_height + ROW_SPACING
        assert_eq!(
            at(MARGIN + 5.0, top + 20.0 + ROW_SPACING + 5.0),
            Some(2)
        );
    }

    /// 间距、窗口外、blur 边距里都不能误命中。
    #[test]
    fn candidate_hit_rejects_gaps_and_outside() {
        let model = hit_model();
        let metrics = hit_metrics();
        let at = |x: f32, y: f32| candidate_item_at(&model, &metrics, x + BLUR_RADIUS, y + BLUR_RADIUS);
        // 两项之间的 COL_SPACING 空隙
        let top = MARGIN + MENU_STRIP_HEIGHT;
        assert_eq!(at(MARGIN + 60.0 + 0.5, top + 5.0), None);
        // blur 边距（内容区之外）
        assert_eq!(candidate_item_at(&model, &metrics, 1.0, 1.0), None);
        // 候选栏下方（面板区）
        assert_eq!(at(MARGIN + 5.0, 100.0), None);
    }
}
