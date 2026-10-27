//! WordCraft's sliders (zoom in the status bar and the Print preview, transparency, Read Aloud
//! speed): egui's slider takes the pointer and the keys, and is drawn as a thin rail with the
//! accent colour up to a round knob, so the rail shows on white panes too.

use egui::{Rect, Response, Stroke, Ui, pos2};

use crate::theme::Tokens;

/// A zoom slider over `range` (on a log scale with `logarithmic`), `width` wide.
pub fn zoom_slider(ui: &mut Ui, value: &mut f32, range: std::ops::RangeInclusive<f32>, logarithmic: bool, width: f32) -> Response {
    slider(ui, value, range, logarithmic, width, 0.0)
}

/// A slider over `range` (on a log scale with `logarithmic`), `width` wide, without a value box
/// (add a `DragValue` after it for one); a positive `step` snaps the value to its multiples.
pub fn slider(ui: &mut Ui, value: &mut f32, range: std::ops::RangeInclusive<f32>, logarithmic: bool, width: f32, step: f64) -> Response {
    let mut slider = egui::Slider::new(value, range.clone())
        .logarithmic(logarithmic)
        .show_value(false)
        .handle_shape(egui::style::HandleShape::Circle)
        .trailing_fill(false);
    if step > 0.0 {
        slider = slider.step_by(step);
    }
    // egui's own rail and knob are drawn invisibly; ours are painted over them.
    let response = ui
        .scope(|ui| {
            ui.spacing_mut().slider_width = width;
            let w = &mut ui.visuals_mut().widgets;
            for s in [&mut w.inactive, &mut w.hovered, &mut w.active] {
                s.bg_fill = egui::Color32::TRANSPARENT;
                s.fg_stroke = Stroke::NONE;
                s.expansion = 0.0;
            }
            ui.add(slider)
        })
        .inner;
    paint(ui, &response, fraction(*value, &range, logarithmic), &Tokens::get(ui.ctx()));
    response
}

/// Where `value` sits along a slider over `range`, 0 to 1, as egui places its knob: on a log scale
/// for a logarithmic slider over positive values.
pub(crate) fn fraction(value: f32, range: &std::ops::RangeInclusive<f32>, logarithmic: bool) -> f32 {
    let (lo, hi) = (*range.start(), *range.end());
    let f = if logarithmic && lo > 0.0 && value > 0.0 { (value.log10() - lo.log10()) / (hi.log10() - lo.log10()) } else { (value - lo) / (hi - lo) };
    if f.is_finite() { f.clamp(0.0, 1.0) } else { 0.0 }
}

/// A thin rail, the accent colour up to the knob, and a round knob outlined in the accent colour
/// while it is under the pointer, dragged or focused.
fn paint(ui: &Ui, response: &Response, fraction: f32, t: &Tokens) {
    let rect = response.rect;
    // The knob's centre travels where egui's round knob would: inset by its radius at both ends.
    let travel = rect.x_range().shrink(rect.height() / 2.5);
    let x = egui::lerp(travel, fraction);
    let y = rect.center().y;
    let p = ui.painter();
    // The rail spans the knob's travel, so at the start no sliver of accent shows beside the knob.
    let rail = Rect::from_min_max(pos2(travel.min, y - 1.5), pos2(travel.max, y + 1.5));
    p.rect_filled(rail, 1.5, t.border_strong);
    if x > rail.min.x {
        p.rect_filled(Rect::from_min_max(rail.min, pos2(x, rail.max.y)), 1.5, t.accent);
    }
    let hot = response.hovered() || response.dragged() || response.has_focus();
    let ring = if hot { t.accent } else { t.input_border };
    p.circle(pos2(x, y), 6.0, t.input, Stroke::new(if hot { 1.5 } else { 1.0 }, ring));
}

#[cfg(test)]
mod tests {
    use super::fraction;

    /// The knob sits at the ends for the range's ends, halfway on the log scale for the
    /// geometric middle, and never off the rail for values outside the range or junk.
    #[test]
    fn the_knob_follows_the_value() {
        let zoom = 0.1..=5.0;
        assert_eq!(fraction(0.1, &zoom, true), 0.0);
        assert_eq!(fraction(5.0, &zoom, true), 1.0);
        let middle = (0.1f32 * 5.0).sqrt();
        assert!((fraction(middle, &zoom, true) - 0.5).abs() < 1e-4);
        assert!((fraction(1.125, &(0.75..=1.5), false) - 0.5).abs() < 1e-6);
        assert_eq!(fraction(50.0, &zoom, true), 1.0);
        assert_eq!(fraction(-1.0, &zoom, true), 0.0);
        assert_eq!(fraction(f32::NAN, &zoom, true), 0.0);
        assert_eq!(fraction(1.0, &(1.0..=1.0), false), 0.0);
    }
}
