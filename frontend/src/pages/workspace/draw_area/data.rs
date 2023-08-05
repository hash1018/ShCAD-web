use wasm_bindgen::JsCast;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, WebGlBuffer, WebGlProgram,
    WebGlRenderingContext as GL, WebGlShader, WheelEvent,
};
use yew::NodeRef;

use crate::{
    algorithm::coordinates_converter::convert_device_to_figure,
    base::{DrawOption, ShouldAction},
    Coordinates,
};

#[derive(Default)]
pub struct DrawAreaData {
    node_ref: NodeRef,
    coordinates: Coordinates,
}

impl DrawAreaData {
    pub fn new() -> Self {
        Self {
            node_ref: NodeRef::default(),
            coordinates: Coordinates::new(),
        }
    }

    pub fn convert_canvas(&self) -> HtmlCanvasElement {
        self.node_ref.cast::<HtmlCanvasElement>().unwrap()
    }

    pub fn convert_gl_context(&self) -> GL {
        let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
        canvas
            .get_context("webgl")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap()
    }

    pub fn convert_2d_context(&self) -> CanvasRenderingContext2d {
        let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
        canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap()
    }

    pub fn coordinates(&self) -> &Coordinates {
        &self.coordinates
    }

    pub fn node_ref(&self) -> NodeRef {
        self.node_ref.clone()
    }

    pub fn set_scroll_pos(&mut self, h_pos: f64, v_pos: f64) {
        self.coordinates.scroll_h_pos = h_pos;
        self.coordinates.scroll_v_pos = v_pos;
    }

    pub fn append_scroll_pos(&mut self, h_pos: f64, v_pos: f64) {
        self.coordinates.scroll_h_pos += h_pos;
        self.coordinates.scroll_v_pos += v_pos;
    }

    pub fn zoom_in(&mut self, event: WheelEvent) -> Option<Vec<ShouldAction>> {
        let device_x = event.offset_x() as f64;
        let device_y = event.offset_y() as f64;
        let (x, y) = convert_device_to_figure(self.coordinates(), device_x, device_y);

        let delta = event.delta_y().abs();

        let zoom = if delta < 10.0 { 0.02 } else { 0.2 };

        if self.coordinates.zoom_rate < 4.0 {
            self.coordinates.zoom_rate += zoom;
        } else {
            return None;
        }

        if self.coordinates.zoom_rate > 4.0 {
            self.coordinates.zoom_rate = 4.0;
        }

        self.coordinates.scroll_v_pos = -1.0 * (self.coordinates.zoom_rate * y) - device_y
            + (self.coordinates.center_y * self.coordinates.zoom_rate);
        self.coordinates.scroll_h_pos = self.coordinates.zoom_rate * x - device_x
            + (self.coordinates.center_x * self.coordinates.zoom_rate);

        Some(vec![ShouldAction::Rerender(DrawOption::DrawAll)])
    }

    pub fn zoom_out(&mut self, event: WheelEvent) -> Option<Vec<ShouldAction>> {
        let device_x = event.offset_x() as f64;
        let device_y = event.offset_y() as f64;
        let (x, y) = convert_device_to_figure(self.coordinates(), device_x, device_y);

        let delta = event.delta_y().abs();

        let zoom = if delta < 10.0 { 0.02 } else { 0.2 };

        if self.coordinates.zoom_rate > 0.5 {
            self.coordinates.zoom_rate -= zoom;
        } else {
            return None;
        }

        if self.coordinates.zoom_rate < 0.5 {
            self.coordinates.zoom_rate = 0.5;
        }

        self.coordinates.scroll_v_pos = -1.0 * (self.coordinates.zoom_rate * y) - device_y
            + (self.coordinates.center_y * self.coordinates.zoom_rate);
        self.coordinates.scroll_h_pos = self.coordinates.zoom_rate * x - device_x
            + (self.coordinates.center_x * self.coordinates.zoom_rate);

        Some(vec![ShouldAction::Rerender(DrawOption::DrawAll)])
    }
}

#[allow(dead_code)]
pub struct WebGLData {
    vertex_shader: WebGlShader,
    fragment_shader: WebGlShader,
    shader_program: WebGlProgram,
    vertex_buffer: WebGlBuffer,
}

impl WebGLData {
    pub fn new(gl: &GL) -> Option<Self> {
        let vert_code = "precision mediump float;

        attribute vec2 a_position;

        void main() {
            gl_Position = vec4(a_position, 0.0, 1.0);
        }";

        let frag_code = "precision mediump float;

        uniform vec4 color;

        void main() {
            gl_FragColor = color;
        }";

        let vertex_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
        gl.shader_source(&vertex_shader, vert_code);
        gl.compile_shader(&vertex_shader);

        let fragment_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
        gl.shader_source(&fragment_shader, frag_code);
        gl.compile_shader(&fragment_shader);

        let shader_program = gl.create_program().unwrap();
        gl.attach_shader(&shader_program, &vertex_shader);
        gl.attach_shader(&shader_program, &fragment_shader);
        gl.link_program(&shader_program);

        gl.use_program(Some(&shader_program));

        let vertex_buffer = gl.create_buffer().unwrap();

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_buffer));

        let position = gl.get_attrib_location(&shader_program, "a_position") as u32;
        gl.vertex_attrib_pointer_with_i32(position, 2, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(position);

        Some(Self {
            vertex_shader,
            fragment_shader,
            shader_program,
            vertex_buffer,
        })
    }

    pub fn shader_program(&self) -> &WebGlProgram {
        &self.shader_program
    }
}
