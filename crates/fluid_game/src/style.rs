//! Neon look for the menus (setup screen, pause dialog): dark translucent
//! panels with a cyan edge and cyan highlights, matching the glowing grid.

use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, Stroke};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

const CYAN: Color32 = Color32::from_rgb(0, 255, 255);
const DEEP_CYAN: Color32 = Color32::from_rgb(0, 110, 130);
const PANEL: Color32 = Color32::from_rgba_premultiplied(6, 6, 14, 225);
const WIDGET: Color32 = Color32::from_rgb(20, 22, 40);
const TEXT: Color32 = Color32::from_rgb(215, 230, 255);

pub struct StylePlugin;

impl Plugin for StylePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, apply_neon_style);
    }
}

/// Style the egui context once, as soon as it exists.
fn apply_neon_style(mut contexts: EguiContexts, mut done: Local<bool>) -> Result {
    if *done {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(TEXT);
    visuals.window_fill = PANEL;
    visuals.panel_fill = PANEL;
    visuals.window_stroke = Stroke::new(1.5, CYAN);
    visuals.extreme_bg_color = WIDGET;
    visuals.selection.bg_fill = DEEP_CYAN;
    visuals.selection.stroke = Stroke::new(1.0, CYAN);
    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.bg_fill = WIDGET;
        widget.weak_bg_fill = WIDGET;
    }
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, CYAN);
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, CYAN);
    visuals.widgets.active.weak_bg_fill = DEEP_CYAN;
    ctx.set_visuals(visuals);
    ctx.all_styles_mut(|style| {
        for font in style.text_styles.values_mut() {
            font.size *= 1.2;
        }
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
    });
    *done = true;
    Ok(())
}
