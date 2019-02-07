// pathfinder/renderer/src/scene.rs
//
// Copyright © 2019 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A set of paths to be rendered.

use crate::builder::{PreparedRenderOptions, PreparedRenderTransform, RenderOptions};
use crate::gpu_data::BuiltObject;
use crate::paint::{ObjectShader, Paint, PaintId, ShaderId};
use crate::tiles::Tiler;
use crate::z_buffer::ZBuffer;
use hashbrown::HashMap;
use pathfinder_geometry::basic::rect::{RectF32, RectI32};
use pathfinder_geometry::outline::Outline;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::fmt::{self, Debug, Formatter};

#[derive(Clone)]
pub struct Scene {
    pub objects: Vec<PathObject>,
    pub paints: Vec<Paint>,
    pub paint_cache: HashMap<Paint, PaintId>,
    pub bounds: RectF32,
    pub view_box: RectF32,
}

impl Scene {
    #[inline]
    pub fn new() -> Scene {
        Scene {
            objects: vec![],
            paints: vec![],
            paint_cache: HashMap::new(),
            bounds: RectF32::default(),
            view_box: RectF32::default(),
        }
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn push_paint(&mut self, paint: &Paint) -> PaintId {
        if let Some(paint_id) = self.paint_cache.get(paint) {
            return *paint_id;
        }

        let paint_id = PaintId(self.paints.len() as u16);
        self.paint_cache.insert(*paint, paint_id);
        self.paints.push(*paint);
        paint_id
    }

    pub fn build_shaders(&self) -> Vec<ObjectShader> {
        self.paints
            .iter()
            .map(|paint| ObjectShader {
                fill_color: paint.color,
            })
            .collect()
    }

    pub fn build_objects_sequentially(&self, options: RenderOptions, z_buffer: &ZBuffer)
                                      -> Vec<BuiltObject> {
        let built_options = options.prepare(self.bounds);
        self.objects
            .iter()
            .enumerate()
            .map(|(object_index, object)| {
                let outline = self.apply_render_options(&object.outline, &built_options);
                let mut tiler = Tiler::new(
                    &outline,
                    self.view_box,
                    object_index as u16,
                    ShaderId(object.paint.0),
                    z_buffer,
                );
                tiler.generate_tiles();
                tiler.built_object
            })
            .collect()
    }

    pub fn build_objects(&self, options: RenderOptions, z_buffer: &ZBuffer) -> Vec<BuiltObject> {
        let built_options = options.prepare(self.bounds);
        self.objects
            .par_iter()
            .enumerate()
            .map(|(object_index, object)| {
                let outline = self.apply_render_options(&object.outline, &built_options);
                let mut tiler = Tiler::new(
                    &outline,
                    self.view_box,
                    object_index as u16,
                    ShaderId(object.paint.0),
                    z_buffer,
                );
                tiler.generate_tiles();
                tiler.built_object
            })
            .collect()
    }

    fn apply_render_options(&self, outline: &Outline, options: &PreparedRenderOptions) -> Outline {
        // FIXME(pcwalton): Don't clone?
        let mut outline = (*outline).clone();
        match options.transform {
            PreparedRenderTransform::Perspective { ref perspective, ref clip_polygon } => {
                outline.clip_against_polygon(clip_polygon);
                outline.apply_perspective(perspective);
            }
            PreparedRenderTransform::Transform2D(ref transform) => {
                outline.transform(transform);
                outline.clip_against_rect(self.view_box);
            }
            PreparedRenderTransform::None => {
                outline.clip_against_rect(self.view_box);
            }
        }

        if !options.dilation.is_zero() {
            outline.dilate(options.dilation);
        }

        outline.prepare_for_tiling(self.view_box);
        outline
    }
}

impl Debug for Scene {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        writeln!(formatter,
                 "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{} {} {} {}\">",
                 self.view_box.origin().x(),
                 self.view_box.origin().y(),
                 self.view_box.size().x(),
                 self.view_box.size().y())?;
        for object in &self.objects {
            let paint = &self.paints[object.paint.0 as usize];
            write!(formatter, "    <path")?;
            if !object.name.is_empty() {
                write!(formatter, " id=\"{}\"", object.name)?;
            }
            writeln!(formatter, " fill=\"{:?}\" d=\"{:?}\" />", paint.color, object.outline)?;
        }
        writeln!(formatter, "</svg>")?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct PathObject {
    outline: Outline,
    paint: PaintId,
    name: String,
    kind: PathObjectKind,
}

#[derive(Clone, Copy, Debug)]
pub enum PathObjectKind {
    Fill,
    Stroke,
}

impl PathObject {
    #[inline]
    pub fn new(outline: Outline, paint: PaintId, name: String, kind: PathObjectKind)
               -> PathObject {
        PathObject {
            outline,
            paint,
            name,
            kind,
        }
    }

    #[inline]
    pub fn outline(&self) -> &Outline {
        &self.outline
    }
}

// TODO(pcwalton): Use a `Point2DI32` here?
#[inline]
pub fn scene_tile_index(tile_x: i32, tile_y: i32, tile_rect: RectI32) -> u32 {
    (tile_y - tile_rect.min_y()) as u32 * tile_rect.size().x() as u32
        + (tile_x - tile_rect.min_x()) as u32
}