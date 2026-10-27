//! Drop-down lists in dialogs, panes and the File page: egui's combo box drawn the WordCraft way.
//! The field keeps the input colour with a chevron at its end, its outline turns the accent
//! colour while the list is open, and the list reads like a menu: roomy rows, a rounded
//! highlight under the pointer, no box around a row, the chosen row tinted.
//!
//! Use [`dropdown`] where `egui::ComboBox::from_id_salt` would go; it takes the same calls.

use egui::{InnerResponse, Rect, Ui, style::WidgetVisuals, vec2};

use crate::icons;
use crate::theme::{self, Tokens};

/// A drop-down with the id `id_salt`, like `egui::ComboBox::from_id_salt`.
pub fn dropdown<'a>(id_salt: impl egui::AsIdSalt) -> Dropdown<'a> {
    Dropdown { combo: egui::ComboBox::from_id_salt(id_salt) }
}

/// A drop-down being built; see [`dropdown`].
#[must_use = "a drop-down shows nothing until `show_ui` is called"]
pub struct Dropdown<'a> {
    combo: egui::ComboBox<'a>,
}

impl<'a> Dropdown<'a> {
    /// The field's least width, as `egui::ComboBox::width`.
    pub fn width(self, width: f32) -> Self {
        Dropdown { combo: self.combo.width(width) }
    }

    /// The list's greatest height before it scrolls, as `egui::ComboBox::height`.
    pub fn height(self, height: f32) -> Self {
        Dropdown { combo: self.combo.height(height) }
    }

    /// What the field shows: the current choice.
    pub fn selected_text(self, text: impl egui::IntoAtoms<'a>) -> Self {
        Dropdown { combo: self.combo.selected_text(text) }
    }

    /// Draws the field, and the list from `contents` while it is open.
    pub fn show_ui<R>(self, ui: &mut Ui, contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<Option<R>> {
        let t = Tokens::get(ui.ctx());
        ui.scope(|ui| {
            theme::field_visuals(ui.visuals_mut(), &t);
            self.combo.icon(paint_chevron).popup_style(egui::style::StyleModifier::new(theme::list_rows)).show_ui(ui, contents)
        })
        .inner
    }
}

/// A small field that opens a menu (`add_contents`): the input colour with a chevron, and a colour
/// swatch before it when given; outlined like a drop-down, in the accent colour while open.
/// `label` names it for screen readers. The menu's result, while it is open.
pub fn menu_field<R>(ui: &mut Ui, label: &str, swatch: Option<egui::Color32>, add_contents: impl FnOnce(&mut Ui) -> R) -> Option<R> {
    let t = Tokens::get(ui.ctx());
    let h = ui.spacing().interact_size.y;
    let w = if swatch.is_some() { 46.0 } else { 30.0 };
    let (rect, resp) = ui.allocate_exact_size(vec2(w, h), egui::Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), label));
    let open = egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&resp));
    let stroke = if open || resp.has_focus() {
        t.accent
    } else if resp.hovered() {
        t.text_dim
    } else {
        t.input_border
    };
    let p = ui.painter();
    p.rect(rect, egui::CornerRadius::same(theme::CONTROL_RADIUS), t.input, egui::Stroke::new(1.0, stroke), egui::StrokeKind::Inside);
    if let Some(c) = swatch {
        let sw = Rect::from_min_size(egui::pos2(rect.min.x + 7.0, rect.center().y - 6.0), vec2(18.0, 12.0));
        p.rect_filled(sw, 2.0, c);
        p.rect_stroke(sw, 2.0, egui::Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
    }
    let chevron = Rect::from_center_size(egui::pos2(rect.max.x - 11.0, rect.center().y), vec2(super::DROPDOWN, super::DROPDOWN));
    icons::paint(p, chevron, if open { "dropup" } else { "dropdown" }, t.text_dim, t.text_dim);
    egui::Popup::menu(&resp).show(add_contents).map(|r| r.inner)
}

/// The label before a field in a form row (`text` already translated), in a column at least
/// `width` wide so the fields of the rows above and below line up; a longer label (a long
/// translation) widens its own row only.
pub fn field_label(ui: &mut Ui, text: &str, width: f32) -> egui::Response {
    let h = ui.spacing().interact_size.y;
    ui.allocate_ui_with_layout(vec2(width, h), egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.set_min_width(width);
        ui.label(text)
    })
    .inner
}

/// The width of the widest of `labels` in the body font: the label column [`field_label`] needs.
pub fn label_column(ui: &Ui, labels: &[&str]) -> f32 {
    let font = egui::TextStyle::Body.resolve(ui.style());
    labels.iter().map(|l| ui.painter().layout_no_wrap((*l).to_string(), font.clone(), egui::Color32::PLACEHOLDER).size().x).fold(0.0, f32::max)
}

/// The chevron at the field's end: down while the list is closed, up while it is open.
fn paint_chevron(ui: &Ui, rect: Rect, _visuals: &WidgetVisuals, open: bool) {
    let t = Tokens::get(ui.ctx());
    let c = if ui.is_enabled() { t.text_dim } else { t.text_disabled };
    let r = Rect::from_center_size(rect.center(), vec2(super::DROPDOWN + 2.0, super::DROPDOWN + 2.0));
    icons::paint(ui.painter(), r, if open { "dropup" } else { "dropdown" }, c, c);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One frame of `build` with the app's light style; the shapes it painted.
    fn painted(build: impl FnOnce(&mut Ui)) -> Vec<egui::epaint::ClippedShape> {
        let ctx = egui::Context::default();
        crate::theme::apply(&ctx, crate::theme::Appearance::Light);
        let mut build = Some(build);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            if let Some(b) = build.take() {
                b(ui);
            }
        });
        let shapes = std::mem::take(&mut out.shapes);
        out.drop_without_applying_deltas();
        shapes
    }

    /// The field is painted in the input colour with the input outline, rounded like the other
    /// controls, with a chevron line rather than egui's filled triangle.
    #[test]
    fn the_field_takes_the_input_look_and_a_chevron() {
        let t = Tokens::light();
        let shapes = painted(|ui| {
            let mut v = 1;
            dropdown("d").selected_text("One").width(120.0).show_ui(ui, |ui| {
                ui.selectable_value(&mut v, 1, "One");
                ui.selectable_value(&mut v, 2, "Two");
            });
        });
        let field = shapes.iter().find_map(|c| match &c.shape {
            egui::Shape::Rect(r) if r.fill == t.input && r.stroke.color == t.input_border => Some(r.clone()),
            _ => None,
        });
        let field = field.unwrap_or_else(|| panic!("no field in the input colour with the input outline"));
        assert_eq!(field.corner_radius, egui::CornerRadius::same(crate::theme::CONTROL_RADIUS));
        assert!(field.rect.width() >= 120.0, "the field keeps its width: {:?}", field.rect);
        // The chevron is a line in the dim text colour, not a filled triangle.
        let dim = egui::epaint::ColorMode::Solid(t.text_dim);
        let chevron = shapes.iter().any(|c| matches!(&c.shape, egui::Shape::Path(p) if !p.closed && p.stroke.color == dim));
        assert!(chevron, "no chevron line in the field");
        let triangle = shapes.iter().any(|c| matches!(&c.shape, egui::Shape::Path(p) if p.closed && p.fill == t.text));
        assert!(!triangle, "egui's filled triangle is still drawn");
    }

    /// A menu field draws a field (input colour, input outline, the controls' corners) with its
    /// colour swatch and a chevron, where Format Shape had a bare "▾" text button; closed, the
    /// menu's contents are not built.
    #[test]
    fn a_menu_field_is_a_field_with_a_swatch_and_a_chevron() {
        let t = Tokens::light();
        let swatch = egui::Color32::from_rgb(0x5B, 0x8D, 0xEF);
        let mut built = false;
        let shapes = painted(|ui| {
            let out = menu_field(ui, "Color", Some(swatch), |_| built = true);
            assert!(out.is_none(), "a closed menu gives nothing");
        });
        assert!(!built, "a closed menu builds nothing");
        let field = shapes.iter().any(|c| {
            matches!(&c.shape, egui::Shape::Rect(r) if r.fill == t.input && r.stroke.color == t.input_border
                && r.corner_radius == egui::CornerRadius::same(crate::theme::CONTROL_RADIUS))
        });
        assert!(field, "no field in the input colour with the input outline");
        assert!(shapes.iter().any(|c| matches!(&c.shape, egui::Shape::Rect(r) if r.fill == swatch)), "no swatch");
        let dim = egui::epaint::ColorMode::Solid(t.text_dim);
        assert!(shapes.iter().any(|c| matches!(&c.shape, egui::Shape::Path(p) if !p.closed && p.stroke.color == dim)), "no chevron");
    }

    /// The list's rows have no outline (the chosen one was boxed in the input border) and the
    /// chosen row is tinted with the accent's `checked` colour.
    #[test]
    fn list_rows_have_no_boxes_and_the_chosen_row_is_tinted() {
        // egui's own default style is the dark one: start from the light visuals.
        let mut style = egui::Style { visuals: egui::Visuals::light(), ..Default::default() };
        crate::theme::list_rows(&mut style);
        let t = Tokens::light();
        let w = &style.visuals.widgets;
        for (name, v) in [("inactive", w.inactive), ("hovered", w.hovered), ("active", w.active), ("open", w.open)] {
            assert_eq!(v.bg_stroke, egui::Stroke::NONE, "{name} rows are outlined");
            assert_eq!(v.corner_radius, egui::CornerRadius::same(crate::theme::ROW_RADIUS), "{name} rows are not rounded");
        }
        assert_eq!(w.hovered.weak_bg_fill, t.hover);
        assert_eq!(style.visuals.selection.bg_fill, t.checked);
        assert_eq!(style.visuals.selection.stroke.color, t.accent_text);
        assert!(style.spacing.button_padding.y >= 4.0, "rows are cramped: {:?}", style.spacing.button_padding);
    }
}
