use super::buttons::DROPDOWN;
use super::combo::drawn_state;
use super::*;

/// The least width of a dialog's OK, Cancel and Close buttons, so a short word still makes a
/// button that is easy to hit and the pair lines up.
pub const DIALOG_BUTTON_W: f32 = 76.0;

/// A dialog's primary button (OK, Save, Close): the accent fill with `on_accent` text, per theme,
/// without the field outline round it.
pub fn primary_button(ui: &Ui, text: impl Into<String>) -> egui::Button<'static> {
    let t = Tokens::get(ui.ctx());
    egui::Button::new(egui::RichText::new(text.into()).color(t.on_accent)).fill(t.accent).stroke(Stroke::NONE).min_size(vec2(DIALOG_BUTTON_W, 0.0))
}

/// A dialog's other buttons (Cancel, Apply…): as wide as [`primary_button`] at least.
pub fn secondary_button(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(text.into()).min_size(vec2(DIALOG_BUTTON_W, 0.0))
}

/// A frameless 22 px icon button (pane close buttons): ribbon-style hover and press feedback.
pub fn icon_button(ui: &mut Ui, icon: &str, tip: &str) -> Response {
    let t = Tokens::get(ui.ctx());
    let (r, resp) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::click());
    let st = drawn_state(ui, &resp, false);
    ui.painter().rect_filled(r, CornerRadius::same(4), st.fill);
    icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(14.0, 14.0)), icon, t.icon, t.accent);
    resp.on_hover_text(tip)
}

/// A text button with a drawn icon in front. Symbols such as ✎ or 💬 aren't in the interface
/// fonts and would draw as empty boxes, so icons are painted (`icons.rs`) rather than typed.
/// It looks and reacts like `ui.button` (see [`drawn_state`]).
pub fn icon_text_button(ui: &mut Ui, icon: &str, text: &str, font: egui::FontId) -> Response {
    icon_text_button_with(ui, icon, text, font, false, true)
}

/// [`icon_text_button`] that opens a menu: the drop-down marker follows the text.
pub fn icon_text_menu_button(ui: &mut Ui, icon: &str, text: &str, font: egui::FontId) -> Response {
    icon_text_button_with(ui, icon, text, font, true, true)
}

/// The icon-only form of [`icon_text_button`] / [`icon_text_menu_button`] for a crowded strip
/// (#2026): `label` is the accessible name and the tooltip, not drawn.
pub fn icon_only_button(ui: &mut Ui, icon: &str, label: &str, font: egui::FontId, marker: bool) -> Response {
    icon_text_button_with(ui, icon, label, font, marker, false).on_hover_text(label)
}

/// Width [`icon_text_button`] (or its menu / icon-only forms) takes.
pub fn icon_text_button_width(ui: &Ui, text: &str, font: egui::FontId, marker: bool, show_text: bool) -> f32 {
    let text_w = if show_text { ui.ctx().fonts_mut(|f| f.layout_no_wrap(text.to_string(), font, Color32::PLACEHOLDER).size().x) } else { 0.0 };
    icon_text_button_size(ui, text_w, marker, show_text)
}

fn icon_text_button_size(ui: &Ui, text_w: f32, marker: bool, show_text: bool) -> f32 {
    let pad = ui.spacing().button_padding;
    let gap = if show_text { 4.0 } else { 0.0 };
    let marker_w = if marker { DROPDOWN + 4.0 } else { 0.0 };
    pad.x + 16.0 + gap + text_w + marker_w + pad.x
}

/// Allocates and paints a themed icon button with optional text, marker and accessible label, and
/// returns its response.
fn icon_text_button_with(ui: &mut Ui, icon: &str, text: &str, font: egui::FontId, marker: bool, show_text: bool) -> Response {
    let t = Tokens::get(ui.ctx());
    let galley = ui.painter().layout_no_wrap(text.to_string(), font, Color32::PLACEHOLDER);
    let pad = ui.spacing().button_padding;
    let icon_w = 16.0;
    let text_w = if show_text { galley.size().x } else { 0.0 };
    let size = vec2(icon_text_button_size(ui, text_w, marker, show_text), galley.size().y.max(icon_w) + 2.0 * pad.y);
    let (r, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), text));
    let st = drawn_state(ui, &resp, true);
    let r = r.expand(st.expansion);
    ui.painter().rect(r, ui.visuals().widgets.inactive.corner_radius, st.fill, st.stroke, egui::StrokeKind::Inside);
    icons::paint(
        ui.painter(),
        Rect::from_center_size(pos2(r.min.x + pad.x + icon_w / 2.0, r.center().y), vec2(icon_w, icon_w)),
        icon,
        t.icon,
        t.accent,
    );
    let text_x = r.min.x + pad.x + icon_w + if show_text { 4.0 } else { 0.0 };
    if show_text {
        ui.painter().galley(pos2(text_x, r.center().y - galley.size().y / 2.0), galley, st.text);
    }
    if marker {
        let c = pos2(text_x + text_w + 4.0 + DROPDOWN / 2.0, r.center().y);
        icons::paint(ui.painter(), Rect::from_center_size(c, vec2(DROPDOWN, DROPDOWN)), "dropdown", st.text, st.text);
    }
    resp
}

/// A drop-down menu item with `icon` before `text`, at least `min_w` wide.
pub fn icon_menu_item(ui: &mut Ui, icon: &str, text: &str, shortcut: &str, min_w: f32) -> Response {
    let t = Tokens::get(ui.ctx());
    let font = egui::TextStyle::Button.resolve(ui.style());
    let galley = ui.painter().layout_no_wrap(text.to_string(), font.clone(), Color32::PLACEHOLDER);
    let sc = (!shortcut.is_empty()).then(|| ui.painter().layout_no_wrap(shortcut.to_string(), font, Color32::PLACEHOLDER));
    let pad = ui.spacing().button_padding;
    let icon_w = 16.0;
    let sc_w = sc.as_ref().map_or(0.0, |g| g.size().x + 3.0 * pad.x);
    let w = (pad.x + icon_w + 6.0 + galley.size().x + sc_w + pad.x).max(min_w).max(ui.available_width());
    let (r, resp) = ui.allocate_exact_size(vec2(w, galley.size().y.max(icon_w) + 2.0 * pad.y), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), text));
    let st = drawn_state(ui, &resp, false);
    ui.painter().rect_filled(r, ui.visuals().widgets.inactive.corner_radius, st.fill);
    let (line, accent) = if ui.is_enabled() { (t.icon, t.accent) } else { (t.text_disabled, t.text_disabled) };
    let text_c = if ui.is_enabled() { st.text } else { t.text_disabled };
    icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + pad.x + icon_w / 2.0, r.center().y), vec2(icon_w, icon_w)), icon, line, accent);
    ui.painter().galley(pos2(r.min.x + pad.x + icon_w + 6.0, r.center().y - galley.size().y / 2.0), galley, text_c);
    if let Some(g) = sc {
        ui.painter().galley(pos2(r.max.x - pad.x - g.size().x, r.center().y - g.size().y / 2.0), g, t.text_disabled);
    }
    resp
}

/// A strip of dialog tabs: the labels side by side over a hairline, the open tab's label in the
/// accent colour with an accent bar under it, the others dimmer with a soft highlight under the
/// pointer. Every tab is measured in the open tab's weight, so switching tabs moves nothing.
/// Returns the index of the tab the user clicked.
pub fn dialog_tabs(ui: &mut Ui, labels: &[String], selected: usize) -> Option<usize> {
    const H: f32 = 28.0;
    /// Room on each side of a label.
    const PAD: f32 = 12.0;
    /// The least room on each side of a label: in a narrow pane the tabs move closer together
    /// rather than run past its edge.
    const MIN_PAD: f32 = 4.0;
    let t = Tokens::get(ui.ctx());
    let text_w: Vec<f32> = labels.iter().map(|l| ui.painter().layout_no_wrap(l.clone(), crate::theme::medium(12.5), t.text).size().x).collect();
    let texts: f32 = text_w.iter().sum();
    let sides = 2.0 * labels.len().max(1) as f32;
    let avail = ui.available_width();
    let pad = if avail.is_finite() { ((avail - texts - 1.0) / sides).clamp(MIN_PAD, PAD) } else { PAD };
    ui.set_min_width(texts + sides * pad + 1.0);
    let origin = ui.cursor().min;
    let width = ui.available_width();
    let base_y = origin.y + H;
    // The hairline under the strip first, so the open tab's bar lies over it.
    ui.painter().hline(origin.x..=origin.x + width, base_y - 0.5, Stroke::new(1.0, t.border));
    let mut x = origin.x;
    let mut clicked = None;
    for (i, label) in labels.iter().enumerate() {
        let on = i == selected;
        let w = text_w.get(i).copied().unwrap_or(0.0) + 2.0 * pad;
        let r = Rect::from_min_max(pos2(x, origin.y), pos2(x + w, base_y));
        let resp = ui.interact(r, ui.scope_id().with(("dialog_tab", i)), Sense::click());
        resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, on, label));
        let hover = ui.ctx().animate_bool_responsive(resp.id.with("hover"), resp.hovered() && !on);
        let p = ui.painter();
        if hover > 0.0 {
            let fill = Color32::TRANSPARENT.lerp_to_gamma(t.hover, hover);
            p.rect_filled(r.shrink2(vec2(2.0, 3.0)), CornerRadius::same(crate::theme::ROW_RADIUS), fill);
        }
        let (font, color) = if on { (crate::theme::medium(12.5), t.accent_text) } else { (regular(12.5), t.text_dim) };
        let galley = p.layout_no_wrap(label.clone(), font, color);
        p.galley(pos2(r.center().x - galley.size().x / 2.0, r.center().y - galley.size().y / 2.0 - 1.0), galley, color);
        if on {
            let bar = Rect::from_min_max(pos2(r.min.x + pad - 2.0, base_y - 2.5), pos2(r.max.x - pad + 2.0, base_y + 0.5));
            p.rect_filled(bar, CornerRadius::same(2), t.accent);
        }
        if resp.clicked() && !on {
            clicked = Some(i);
        }
        x += w;
    }
    ui.allocate_space(vec2(width, H + 1.0));
    clicked
}

/// [`dialog_tabs`] for English `labels` (translated here) whose open tab is the index `tab`.
pub fn dialog_tabs_index(ui: &mut Ui, labels: &[&str], tab: &mut u8) {
    let shown: Vec<String> = labels.iter().map(|l| tl!(l).to_string()).collect();
    if let Some(i) = dialog_tabs(ui, &shown, usize::from(*tab))
        && let Ok(i) = u8::try_from(i)
    {
        *tab = i;
    }
}

/// A small arrow or "more" button of a gallery strip (the Equation tab's symbols), like the
/// Styles gallery's: the icon on a rounded highlight under the pointer, greyed out and inert while
/// `enabled` is false. `name` is its accessible name (already translated).
pub fn gallery_arrow(ui: &mut Ui, icon: &str, enabled: bool, name: &str) -> Response {
    let t = Tokens::get(ui.ctx());
    let (r, resp) = ui.allocate_exact_size(vec2(16.0, 22.0), if enabled { Sense::click() } else { Sense::hover() });
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, name));
    if enabled && resp.hovered() {
        ui.painter().rect_filled(r, CornerRadius::same(3), t.hover);
    }
    let c = if enabled { t.icon } else { t.text_disabled };
    icons::paint(ui.painter(), r.shrink2(vec2(1.0, 4.0)), icon, c, c);
    resp
}

/// A row that shows or hides more options below it (Find's Search Options): a chevron pointing
/// right while they are hidden and down while they show, then `label` (already translated).
pub fn disclosure(ui: &mut Ui, label: &str, open: bool) -> Response {
    let t = Tokens::get(ui.ctx());
    let font = egui::TextStyle::Body.resolve(ui.style());
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, t.text);
    let (r, resp) = ui.allocate_exact_size(vec2(galley.size().x + 26.0, ui.spacing().interact_size.y), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, ui.is_enabled(), open, label));
    if resp.hovered() {
        ui.painter().rect_filled(r, CornerRadius::same(crate::theme::ROW_RADIUS), t.hover);
    }
    let chevron = Rect::from_center_size(pos2(r.min.x + 10.0, r.center().y), vec2(12.0, 12.0));
    icons::paint(ui.painter(), chevron, if open { "chevronDown" } else { "chevronRight" }, t.icon, t.icon);
    ui.painter().galley(pos2(r.min.x + 20.0, r.center().y - galley.size().y / 2.0), galley, t.text);
    resp
}
