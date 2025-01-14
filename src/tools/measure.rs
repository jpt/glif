use MFEKmath::Vector;
use skulpin::skia_safe::{AutoCanvasRestore, Canvas, Paint, Path, Point, TextBlob, dash_path_effect};

use crate::user_interface::Interface;
use crate::renderer::constants;
use crate::editor::Editor;

use super::prelude::*;

use crate::renderer::string::{ POINTFONTS, POINTFONTSIZE, pointfont_from_size_and_factor};
#[derive(Clone)]
pub struct Measure {
    measure_from: Option<(f32, f32)>
}

impl Tool for Measure {
    fn handle_event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        match event {
            EditorEvent::MouseEvent { event_type, meta } => {
                match event_type {
                    //MouseEventType::Moved => { self.mouse_moved(v, position, meta) }
                    MouseEventType::Pressed => { self.mouse_pressed(v, meta) }
                    MouseEventType::Released => { self.mouse_released(v, meta) }
                    _ => {}
                }
            }
            EditorEvent::Draw { skia_canvas } => {
                self.draw_line(i, skia_canvas);
            }
            _ => {}
        }
    }
}

impl Measure {
    pub fn new() -> Self {
        Self {
            measure_from: None
        }
    }
    
    fn mouse_pressed(&mut self, _v: &Editor ,meta: MouseInfo) {
        self.measure_from = Some(meta.position);
    }

    fn mouse_released(&mut self, _v: &Editor, _meta: MouseInfo) {
        self.measure_from = None;
    }

    fn draw_line(&self, i: &Interface, canvas: &mut Canvas) {
        let mut path = Path::new();
        let mut paint = Paint::default();
        let factor = i.viewport.factor;
        
        if let Some(measure_from) = self.measure_from {
            let skpath_start = Point::new(measure_from.0 as f32, measure_from.1 as f32);
            let skpath_end = Point::new(i.mouse_info.position.0 as f32, i.mouse_info.position.1 as f32);
            
            let start_vec = Vector::from_skia_point(&skpath_start);
            let end_vec = Vector::from_skia_point(&skpath_end);
            let halfway = start_vec.lerp(end_vec, 0.5);
            let unit_vec = (end_vec - start_vec).normalize();
            let angle = f64::atan2(unit_vec.y, unit_vec.x);
            let distance = start_vec.distance(end_vec) * (1. / factor) as f64;

            path.move_to(skpath_start);
            path.line_to(skpath_end);
            path.close();
            paint.set_color(constants::MEASURE_STROKE);
            paint.set_style(skulpin::skia_safe::PaintStyle::Stroke);
            paint.set_stroke_width(OUTLINE_STROKE_THICKNESS * (1. / factor));
            let dash_offset = (1. / factor) * 5.;
            paint.set_path_effect(dash_path_effect::new(&[dash_offset, dash_offset], 0.0));
            canvas.draw_path(&path, &paint);

            draw_measure_string(i, (halfway.x as f32, halfway.y as f32), angle as f32, distance.to_string().as_str(), canvas);
        }
    }
}


pub fn draw_measure_string(i: &Interface, at: (f32, f32), angle: f32, s: &str, canvas: &mut Canvas) {
    let mut arc = AutoCanvasRestore::guard(canvas, true);
    let factor = i.viewport.factor;
    let mut paint = Paint::default();
    paint.set_color(constants::MEASURE_STROKE);
    paint.set_anti_alias(true);

    let (blob, rect) = {
        POINTFONTS.with(|f| {
            let mut hm = f.borrow_mut();
            let f = hm.get(&((POINTFONTSIZE * 1. / factor).round() as usize));
            let font = match f {
                Some(fon) => fon,
                None => {
                    hm.insert(
                        (POINTFONTSIZE * 1. / factor).round() as usize,
                        pointfont_from_size_and_factor(POINTFONTSIZE, factor),
                    );
                    hm.get(&((POINTFONTSIZE * 1. / factor).round() as usize))
                        .unwrap()
                }
            };

            let blob = TextBlob::from_str(s, font).expect(&format!("Failed to shape {}", s));
            let (_, rect) = font.measure_str(s, Some(&paint));
            (blob, rect)
        })
    };

    let center_at = (
        at.0 - rect.width() / 2.,
        at.1 - rect.height() / 2.,
    );
    println!("{0}", angle);
    arc.rotate(angle.to_degrees(), Some(at.into()));
    arc.draw_text_blob(&blob, center_at, &paint);
}
