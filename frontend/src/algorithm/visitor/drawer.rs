use lib::{
    common::Color,
    figure::{leaf::line::Line, Visitor},
};
use web_sys::{CanvasRenderingContext2d, WebGlProgram, WebGlRenderingContext};

use crate::{
    algorithm::{coordinates_converter::convert_figure_to_device, math::caculate_rectangle},
    Coordinates,
};

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
        let start = convert_figure_to_device(self.coordinates, line.start_x(), line.start_y());
        let end = convert_figure_to_device(self.coordinates, line.end_x(), line.end_y());

        draw_line(start, end, &line.color(), self.context);
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

pub struct SelectedDrawer<'a> {
    context: &'a CanvasRenderingContext2d,
    coordinates: &'a Coordinates,
    single_mode: bool,
}

impl<'a> SelectedDrawer<'a> {
    pub fn new(
        context: &'a CanvasRenderingContext2d,
        coordinates: &'a Coordinates,
        single_mode: bool,
    ) -> Self {
        Self {
            context,
            coordinates,
            single_mode,
        }
    }

    pub fn draw_total_selected_rect(
        &self,
        figure_top_left: (f64, f64),
        figure_width: f64,
        figure_height: f64,
    ) {
        let color = Color::new(0, 0, 153, 255);
        let top_left =
            convert_figure_to_device(self.coordinates, figure_top_left.0, figure_top_left.1);
        let bottom_right = convert_figure_to_device(
            self.coordinates,
            figure_top_left.0 + figure_width,
            figure_top_left.1 - figure_height,
        );

        draw_rectangle(
            top_left,
            bottom_right.0 - top_left.0,
            bottom_right.1 - top_left.1,
            &color,
            self.context,
        );
    }
}

impl Visitor for SelectedDrawer<'_> {
    fn visit_line(&self, line: &mut Line) {
        let start = convert_figure_to_device(self.coordinates, line.start_x(), line.start_y());
        let end = convert_figure_to_device(self.coordinates, line.end_x(), line.end_y());

        let color = Color::new(135, 206, 235, 255);

        if self.single_mode {
            fill_circle(start, 6.0, &color, self.context);
            fill_circle(end, 6.0, &color, self.context);
        } else {
            let (top_left, width, height) = caculate_rectangle(start, end, true);
            draw_rectangle(top_left, width, height, &color, self.context);
        }
    }
}

pub struct SelectedByAnotherUserDrawer<'a> {
    context: &'a CanvasRenderingContext2d,
    coordinates: &'a Coordinates,
    color: Color,
}

impl<'a> SelectedByAnotherUserDrawer<'a> {
    pub fn new(
        context: &'a CanvasRenderingContext2d,
        coordinates: &'a Coordinates,
        color: Color,
    ) -> Self {
        Self {
            context,
            coordinates,
            color,
        }
    }
}

impl Visitor for SelectedByAnotherUserDrawer<'_> {
    fn visit_line(&self, line: &mut Line) {
        let start = convert_figure_to_device(self.coordinates, line.start_x(), line.start_y());
        let end = convert_figure_to_device(self.coordinates, line.end_x(), line.end_y());

        let (top_left, width, height) = caculate_rectangle(start, end, true);
        draw_rectangle(top_left, width, height, &self.color, self.context);
    }
}

fn draw_line(
    start: (f64, f64),
    end: (f64, f64),
    color: &Color,
    context: &CanvasRenderingContext2d,
) {
    let color_text = format!("rgb({0},{1},{2})", color.r, color.g, color.b);
    context.set_stroke_style(&color_text.into());

    context.begin_path();
    context.move_to(start.0, start.1);
    context.line_to(end.0, end.1);
    context.close_path();
    context.stroke();
}

fn fill_circle(center: (f64, f64), radius: f64, color: &Color, context: &CanvasRenderingContext2d) {
    let color_text = format!("rgb({0},{1},{2})", color.r, color.g, color.b);
    context.set_fill_style(&color_text.into());
    context.begin_path();
    context.arc(center.0, center.1, radius, 0.0, 360.0).unwrap();
    context.close_path();
    context.fill();
}

fn draw_rectangle(
    top_left: (f64, f64),
    width: f64,
    height: f64,
    color: &Color,
    context: &CanvasRenderingContext2d,
) {
    let color_text = format!("rgb({0},{1},{2})", color.r, color.g, color.b);
    context.set_stroke_style(&color_text.into());
    context.begin_path();
    context.rect(top_left.0, top_left.1, width, height);
    context.close_path();
    context.stroke();
}
