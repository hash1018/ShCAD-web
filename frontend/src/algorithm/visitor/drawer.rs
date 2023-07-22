use lib::figure::{leaf::line::Line, Visitor};
use web_sys::{CanvasRenderingContext2d, WebGlProgram, WebGlRenderingContext};

use crate::{algorithm::coordinates_converter::convert_figure_to_device, Coordinates};

pub struct Drawer<'a> {
    context: &'a CanvasRenderingContext2d,
    coordinates: &'a Coordinates,
}

impl<'a> Drawer<'a> {
    pub fn new(context: &'a CanvasRenderingContext2d, coordinates: &'a Coordinates) -> Self {
        Self {
            context,
            coordinates,
        }
    }
}

impl Visitor for Drawer<'_> {
    fn visit_line(&self, line: &mut Line) {
        let (start_x, start_y) =
            convert_figure_to_device(self.coordinates, line.start_x(), line.start_y());
        let (end_x, end_y) = convert_figure_to_device(self.coordinates, line.end_x(), line.end_y());

        self.context.begin_path();
        self.context.move_to(start_x, start_y);
        self.context.line_to(end_x, end_y);
        self.context.close_path();
        self.context.stroke();
    }
}

pub struct DrawerGL<'a> {
    gl: &'a WebGlRenderingContext,
    shader_program: &'a WebGlProgram,
}

impl<'a> DrawerGL<'a> {
    pub fn new(gl: &'a WebGlRenderingContext, shader_program: &'a WebGlProgram) -> Self {
        Self { gl, shader_program }
    }
}

impl Visitor for DrawerGL<'_> {
    fn visit_line(&self, line: &mut Line) {
        let vectices: Vec<f32> = vec![
            line.start_x() as f32,
            line.start_y() as f32,
            line.end_x() as f32,
            line.end_y() as f32,
        ];
        let verts = js_sys::Float32Array::from(vectices.as_slice());
        self.gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &verts,
            WebGlRenderingContext::STATIC_DRAW,
        );

        let color = self.gl.get_uniform_location(self.shader_program, "color");
        let rgba = line.color();
        self.gl.uniform4f(
            color.as_ref(),
            rgba.r as f32 / 255.0,
            rgba.g as f32 / 255.0,
            rgba.b as f32 / 255.0,
            rgba.a as f32 / 255.0,
        );

        self.gl.draw_arrays(WebGlRenderingContext::LINES, 0, 2);
    }
}
