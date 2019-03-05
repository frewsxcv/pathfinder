// pathfinder/demo/src/ui.rs
//
// Copyright © 2019 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::Options;
use nfd::Response;
use pathfinder_geometry::basic::point::Point2DI32;
use pathfinder_geometry::basic::rect::RectI32;
use pathfinder_gpu::{Device, Resources};
use pathfinder_renderer::gpu::debug::DebugUI;
use pathfinder_ui::{BUTTON_HEIGHT, BUTTON_TEXT_OFFSET, BUTTON_WIDTH, PADDING, SWITCH_SIZE};
use pathfinder_ui::{TEXT_COLOR, UIEvent, WINDOW_COLOR};
use std::f32::consts::PI;
use std::path::PathBuf;

const SLIDER_WIDTH: i32 = 360;
const SLIDER_HEIGHT: i32 = 48;
const SLIDER_TRACK_HEIGHT: i32 = 24;
const SLIDER_KNOB_WIDTH: i32 = 12;
const SLIDER_KNOB_HEIGHT: i32 = 48;

const EFFECTS_PANEL_WIDTH: i32 = 550;
const EFFECTS_PANEL_HEIGHT: i32 = BUTTON_HEIGHT * 3 + PADDING * 4;

const OPEN_BUTTON_X:       i32 = PADDING + (BUTTON_WIDTH + PADDING) * 1;
const SCREENSHOT_BUTTON_X: i32 = PADDING + (BUTTON_WIDTH + PADDING) * 2;
const THREE_D_SWITCH_X:    i32 = PADDING + (BUTTON_WIDTH + PADDING) * 3;
const BACKGROUND_SWITCH_X: i32 = PADDING + (BUTTON_WIDTH + PADDING) * 3 + PADDING + SWITCH_SIZE;

const ROTATE_PANEL_X: i32 = PADDING + (BUTTON_WIDTH + PADDING) * 3 + (PADDING + SWITCH_SIZE) * 2;
const ROTATE_PANEL_WIDTH: i32 = SLIDER_WIDTH + PADDING * 2;
const ROTATE_PANEL_HEIGHT: i32 = PADDING * 2 + SLIDER_HEIGHT;

static EFFECTS_PNG_NAME:    &'static str = "demo-effects";
static OPEN_PNG_NAME:       &'static str = "demo-open";
static ROTATE_PNG_NAME:     &'static str = "demo-rotate";
static ZOOM_IN_PNG_NAME:    &'static str = "demo-zoom-in";
static ZOOM_OUT_PNG_NAME:   &'static str = "demo-zoom-out";
static BG_LIGHT_PNG_NAME:   &'static str = "demo-bg-light";
static BG_DARK_PNG_NAME:    &'static str = "demo-bg-dark";
static SCREENSHOT_PNG_NAME: &'static str = "demo-screenshot";

pub struct DemoUI<D> where D: Device {
    effects_texture: D::Texture,
    open_texture: D::Texture,
    rotate_texture: D::Texture,
    zoom_in_texture: D::Texture,
    zoom_out_texture: D::Texture,
    bg_light_texture: D::Texture,
    bg_dark_texture: D::Texture,
    screenshot_texture: D::Texture,

    effects_panel_visible: bool,
    rotate_panel_visible: bool,

    pub three_d_enabled: bool,
    pub dark_background_enabled: bool,
    pub gamma_correction_effect_enabled: bool,
    pub stem_darkening_effect_enabled: bool,
    pub subpixel_aa_effect_enabled: bool,
    pub rotation: i32,
}

impl<D> DemoUI<D> where D: Device {
    pub fn new(device: &D, resources: &Resources, options: Options) -> DemoUI<D> {
        let effects_texture = device.create_texture_from_png(resources, EFFECTS_PNG_NAME);
        let open_texture = device.create_texture_from_png(resources, OPEN_PNG_NAME);
        let rotate_texture = device.create_texture_from_png(resources, ROTATE_PNG_NAME);
        let zoom_in_texture = device.create_texture_from_png(resources, ZOOM_IN_PNG_NAME);
        let zoom_out_texture = device.create_texture_from_png(resources, ZOOM_OUT_PNG_NAME);
        let bg_light_texture = device.create_texture_from_png(resources, BG_LIGHT_PNG_NAME);
        let bg_dark_texture = device.create_texture_from_png(resources, BG_DARK_PNG_NAME);
        let screenshot_texture = device.create_texture_from_png(resources, SCREENSHOT_PNG_NAME);

        DemoUI {
            effects_texture,
            open_texture,
            rotate_texture,
            zoom_in_texture,
            zoom_out_texture,
            bg_light_texture,
            bg_dark_texture,
            screenshot_texture,

            effects_panel_visible: false,
            rotate_panel_visible: false,

            three_d_enabled: options.three_d,
            dark_background_enabled: true,
            gamma_correction_effect_enabled: false,
            stem_darkening_effect_enabled: false,
            subpixel_aa_effect_enabled: false,
            rotation: SLIDER_WIDTH / 2,
        }
    }

    fn rotation(&self) -> f32 {
        (self.rotation as f32 / SLIDER_WIDTH as f32 * 2.0 - 1.0) * PI
    }

    pub fn update(&mut self,
                  device: &D,
                  debug_ui: &mut DebugUI<D>,
                  event: &mut UIEvent,
                  action: &mut UIAction) {
        let bottom = debug_ui.ui.framebuffer_size().y() - PADDING;

        // Draw effects button.
        let effects_button_position = Point2DI32::new(PADDING, bottom - BUTTON_HEIGHT);
        if debug_ui.ui.draw_button(device, event, effects_button_position, &self.effects_texture) {
            self.effects_panel_visible = !self.effects_panel_visible;
        }

        // Draw open button.
        let lower_button_y = bottom - BUTTON_HEIGHT;
        let open_button_position = Point2DI32::new(OPEN_BUTTON_X, lower_button_y);
        if debug_ui.ui.draw_button(device, event, open_button_position, &self.open_texture) {
            if let Ok(Response::Okay(file)) = nfd::open_file_dialog(Some("svg"), None) {
                *action = UIAction::OpenFile(PathBuf::from(file));
            }
        }

        // Draw screenshot button.
        let screenshot_button_position = Point2DI32::new(SCREENSHOT_BUTTON_X, lower_button_y);
        if debug_ui.ui.draw_button(device,
                                   event,
                                   screenshot_button_position,
                                   &self.screenshot_texture) {
            if let Ok(Response::Okay(file)) = nfd::open_save_dialog(Some("png"), None) {
                *action = UIAction::TakeScreenshot(PathBuf::from(file));
            }
        }

        // Draw 3D switch.
        let three_d_switch_origin = Point2DI32::new(THREE_D_SWITCH_X, lower_button_y);
        self.three_d_enabled = debug_ui.ui.draw_text_switch(device,
                                                            event,
                                                            three_d_switch_origin,
                                                            "2D",
                                                            "3D",
                                                            self.three_d_enabled);

        // Draw background switch.
        let background_switch_origin = Point2DI32::new(BACKGROUND_SWITCH_X, lower_button_y);
        self.dark_background_enabled = debug_ui.ui.draw_image_switch(device,
                                                                     event,
                                                                     background_switch_origin,
                                                                     &self.bg_light_texture,
                                                                     &self.bg_dark_texture,
                                                                     self.dark_background_enabled);

        // Draw rotate and zoom buttons, if applicable.
        if !self.three_d_enabled {
            let rotate_button_y = bottom - BUTTON_HEIGHT;
            let rotate_button_position = Point2DI32::new(ROTATE_PANEL_X, rotate_button_y);
            if debug_ui.ui.draw_button(device,
                                       event,
                                       rotate_button_position,
                                       &self.rotate_texture) {
                self.rotate_panel_visible = !self.rotate_panel_visible;
            }

            let zoom_in_button_x = ROTATE_PANEL_X + BUTTON_WIDTH + PADDING;
            let zoom_in_button_position = Point2DI32::new(zoom_in_button_x, rotate_button_y);
            if debug_ui.ui.draw_button(device,
                                       event,
                                       zoom_in_button_position,
                                       &self.zoom_in_texture) {
                *action = UIAction::ZoomIn;
            }

            let zoom_out_button_x = ROTATE_PANEL_X + (BUTTON_WIDTH + PADDING) * 2;
            let zoom_out_button_position = Point2DI32::new(zoom_out_button_x, rotate_button_y);
            if debug_ui.ui.draw_button(device,
                                       event,
                                       zoom_out_button_position,
                                       &self.zoom_out_texture) {
                *action = UIAction::ZoomOut;
            }
        }

        // Draw effects panel, if necessary.
        self.draw_effects_panel(device, debug_ui, event);

        // Draw rotate panel, if necessary.
        self.draw_rotate_panel(device, debug_ui, event, action);
    }

    fn draw_effects_panel(&mut self, device: &D, debug_ui: &mut DebugUI<D>, event: &mut UIEvent) {
        if !self.effects_panel_visible {
            return;
        }

        let bottom = debug_ui.ui.framebuffer_size().y() - PADDING;
        let effects_panel_y = bottom - (BUTTON_HEIGHT + PADDING + EFFECTS_PANEL_HEIGHT);
        debug_ui.ui.draw_solid_rounded_rect(device,
                                            RectI32::new(Point2DI32::new(PADDING, effects_panel_y),
                                                         Point2DI32::new(EFFECTS_PANEL_WIDTH,
                                                                         EFFECTS_PANEL_HEIGHT)),
                                            WINDOW_COLOR);

        self.gamma_correction_effect_enabled =
            self.draw_effects_switch(device,
                                     debug_ui,
                                     event,
                                     "Gamma Correction",
                                     0,
                                     effects_panel_y,
                                     self.gamma_correction_effect_enabled);
        self.stem_darkening_effect_enabled =
            self.draw_effects_switch(device,
                                     debug_ui,
                                     event,
                                     "Stem Darkening",
                                     1,
                                     effects_panel_y,
                                     self.stem_darkening_effect_enabled);
        self.subpixel_aa_effect_enabled =
            self.draw_effects_switch(device,
                                     debug_ui,
                                     event,
                                     "Subpixel AA",
                                     2,
                                     effects_panel_y,
                                     self.subpixel_aa_effect_enabled);

    }

    fn draw_rotate_panel(&mut self,
                         device: &D,
                         debug_ui: &mut DebugUI<D>,
                         event: &mut UIEvent,
                         action: &mut UIAction) {
        if !self.rotate_panel_visible {
            return;
        }

        let bottom = debug_ui.ui.framebuffer_size().y() - PADDING;
        let rotate_panel_y = bottom - (BUTTON_HEIGHT + PADDING + ROTATE_PANEL_HEIGHT);
        let rotate_panel_origin = Point2DI32::new(ROTATE_PANEL_X, rotate_panel_y);
        let rotate_panel_size = Point2DI32::new(ROTATE_PANEL_WIDTH, ROTATE_PANEL_HEIGHT);
        debug_ui.ui.draw_solid_rounded_rect(device,
                                            RectI32::new(rotate_panel_origin, rotate_panel_size),
                                            WINDOW_COLOR);

        let (widget_x, widget_y) = (ROTATE_PANEL_X + PADDING, rotate_panel_y + PADDING);
        let widget_rect = RectI32::new(Point2DI32::new(widget_x, widget_y),
                                       Point2DI32::new(SLIDER_WIDTH, SLIDER_KNOB_HEIGHT));
        if let Some(position) = event.handle_mouse_down_or_dragged_in_rect(widget_rect) {
            self.rotation = position.x();
            *action = UIAction::Rotate(self.rotation());
        }

        let slider_track_y = rotate_panel_y + PADDING + SLIDER_KNOB_HEIGHT / 2 -
            SLIDER_TRACK_HEIGHT / 2;
        let slider_track_rect =
            RectI32::new(Point2DI32::new(widget_x, slider_track_y),
                         Point2DI32::new(SLIDER_WIDTH, SLIDER_TRACK_HEIGHT));
        debug_ui.ui.draw_rect_outline(device, slider_track_rect, TEXT_COLOR);

        let slider_knob_x = widget_x + self.rotation - SLIDER_KNOB_WIDTH / 2;
        let slider_knob_rect =
            RectI32::new(Point2DI32::new(slider_knob_x, widget_y),
                         Point2DI32::new(SLIDER_KNOB_WIDTH, SLIDER_KNOB_HEIGHT));
        debug_ui.ui.draw_solid_rect(device, slider_knob_rect, TEXT_COLOR);
    }

    fn draw_effects_switch(&self,
                           device: &D,
                           debug_ui: &mut DebugUI<D>,
                           event: &mut UIEvent,
                           text: &str,
                           index: i32,
                           window_y: i32,
                           value: bool)
                           -> bool {
        let text_x = PADDING * 2;
        let text_y = window_y + PADDING + BUTTON_TEXT_OFFSET + (BUTTON_HEIGHT + PADDING) * index;
        debug_ui.ui.draw_text(device, text, Point2DI32::new(text_x, text_y), false);

        let switch_x = PADDING + EFFECTS_PANEL_WIDTH - (SWITCH_SIZE + PADDING);
        let switch_y = window_y + PADDING + (BUTTON_HEIGHT + PADDING) * index;
        debug_ui.ui.draw_text_switch(device,
                                     event,
                                     Point2DI32::new(switch_x, switch_y),
                                     "Off",
                                     "On",
                                     value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UIAction {
    None,
    OpenFile(PathBuf),
    TakeScreenshot(PathBuf),
    ZoomIn,
    ZoomOut,
    Rotate(f32),
}