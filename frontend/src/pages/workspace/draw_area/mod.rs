use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use lib::{
    common::Color,
    figure::{leaf::line::Line, Figure},
};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent, MouseEvent, WebGlProgram,
    WebGlRenderingContext as GL, WheelEvent,
};
use yew::{html, Callback, Component, Context, Properties};

use crate::{
    algorithm::{
        coordinates_converter::{convert_device_to_figure, convert_figure_to_webgl},
        draw_mode::{pan_mode::PanMode, select_mode::SelectMode, DrawMode},
        visitor::drawer::{Drawer, DrawerGL},
    },
    base::{DrawModeType, DrawOption, ShouldAction},
};

use self::{
    data::{DrawAreaData, WebGLData},
    global_event_handler::GlobalEventHandler,
    mouse_tracker::MouseTracker,
};

use super::{
    data::{FigureList, SharedUsers},
    workspace::ChildRequestType,
    UpdateReason,
};

pub mod data;
mod global_event_handler;
mod mouse_tracker;

pub enum DrawAreaMessage {
    MouseDown(MouseEvent),
    MouseMove(MouseEvent), //This message occurs when mousemove event is triggered.
    MouseUp(MouseEvent),
    KeyDown(KeyboardEvent),
    Wheel(WheelEvent),
    //When the mouse position is checked at intervals by a timer,
    //this message occurs if the position has changed
    MousePositionChanged(VecDeque<(f64, f64)>),
    VisibilityChange(bool),
}

#[derive(Clone, PartialEq, Properties)]
pub struct DrawAreaProps {
    pub handler: Callback<ChildRequestType>,
    pub current_mode: DrawModeType,
    pub figures: Rc<FigureList>,
    pub update_reason: Option<UpdateReason>,
    pub shared_users: Rc<SharedUsers>,
}

pub struct DrawArea {
    data: DrawAreaData,
    current_mode: Box<dyn DrawMode>,
    pan_mode: Option<PanMode>,
    webgl_data: Option<WebGLData>,
    global_event_handler: GlobalEventHandler,
    draw_option: DrawOption,
    mouse_tracker: MouseTracker,
    animation_handle: Rc<RefCell<Option<i32>>>,
}

impl Component for DrawArea {
    type Message = DrawAreaMessage;
    type Properties = DrawAreaProps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let data = DrawAreaData::new();
        let current_mode = SelectMode::new();

        let mut global_event_handler = GlobalEventHandler::new();
        global_event_handler.init(ctx);

        let link = ctx.link().clone();

        let mut mouse_tracker = MouseTracker::new();
        mouse_tracker.run(link);

        DrawArea {
            data,
            current_mode: Box::new(current_mode),
            pan_mode: None,
            webgl_data: None,
            global_event_handler,
            draw_option: DrawOption::DrawAll,
            mouse_tracker,
            animation_handle: Rc::new(RefCell::new(None)),
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.global_event_handler.deinit();
        self.mouse_tracker.stop();
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let update_reason = &ctx.props().update_reason;

        if let Some(update_reason) = update_reason {
            match update_reason {
                UpdateReason::ChangeMode => {
                    self.current_mode = ctx.props().current_mode.into();

                    if self.data.take_preview().is_some() {
                        self.draw_option = DrawOption::DrawAll;
                    } else {
                        self.draw_option = DrawOption::Remain;
                    }
                    return true;
                }
                UpdateReason::FigureAdded | UpdateReason::GetCurrentFigures => {
                    self.draw_option = DrawOption::DrawAll;
                    return true;
                }
                UpdateReason::MousePositionChanged => {
                    self.draw_option = DrawOption::DrawAll;
                    return true;
                }
                UpdateReason::UserLeft => {
                    self.draw_option = DrawOption::DrawAll;
                    return true;
                }
                _ => return false,
            }
        }

        false
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        let canvas = self.data.convert_canvas();
        let context = self.data.convert_2d_context();
        let props = ctx.props();

        if let Some(handle) = self.animation_handle.borrow_mut().take() {
            web_sys::window()
                .unwrap()
                .cancel_animation_frame(handle)
                .unwrap();
        }

        self.render_2d_context(canvas, context, props);

        //let canvas = self.data.convert_canvas();
        //let gl: GL = self.data.convert_gl_context();
        //self.render_gl(gl, canvas);
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        let should_action = match msg {
            DrawAreaMessage::MouseDown(event) => match event.button() {
                0 => self
                    .current_mode
                    .mouse_left_press_event(event, &mut self.data),
                1 => {
                    let mut pan_mode = PanMode::new();
                    let should_action = pan_mode.mouse_press_event(event, &mut self.data);
                    self.pan_mode = Some(pan_mode);
                    should_action
                }
                _ => None,
            },
            DrawAreaMessage::MouseMove(event) => {
                let (x, y) = convert_device_to_figure(
                    self.data.coordinates(),
                    event.offset_x() as f64,
                    event.offset_y() as f64,
                );
                self.mouse_tracker.set_current_pos(x, y);

                if let Some(mut pan_mode) = self.pan_mode.take() {
                    let should_action = pan_mode.mouse_mouse_event(event, &mut self.data);
                    self.pan_mode = Some(pan_mode);
                    should_action
                } else {
                    self.current_mode.mouse_mouse_event(event, &mut self.data)
                }
            }
            DrawAreaMessage::MouseUp(event) => {
                if self.pan_mode.take().is_none() {
                    self.current_mode.mouse_release_event(event, &mut self.data)
                } else {
                    Some(ShouldAction::Rerender(DrawOption::Remain))
                }
            }
            DrawAreaMessage::KeyDown(event) => {
                //Esc key down.
                if event.key_code() == 27 {
                    if self.current_mode.get_type() != DrawModeType::SelectMode {
                        Some(ShouldAction::BackToSelect)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            DrawAreaMessage::Wheel(event) => {
                if event.ctrl_key() || event.meta_key() {
                    if event.delta_y() < 0.0 {
                        self.data.zoom_in(event)
                    } else {
                        self.data.zoom_out(event)
                    }
                } else {
                    self.data
                        .append_scroll_pos(event.delta_x(), event.delta_y());

                    Some(ShouldAction::Rerender(DrawOption::DrawAll))
                }
            }
            DrawAreaMessage::MousePositionChanged(queue) => {
                Some(ShouldAction::NotifyMousePositionChanged(queue))
            }
            DrawAreaMessage::VisibilityChange(visible) => {
                if visible {
                    ctx.props().shared_users.clear_mouse_position_queue();
                    Some(ShouldAction::Rerender(DrawOption::DrawAll))
                } else {
                    None
                }
            }
        };

        if let Some(should_action) = should_action {
            match should_action {
                ShouldAction::BackToSelect => {
                    ctx.props()
                        .handler
                        .emit(ChildRequestType::ChangeMode(DrawModeType::SelectMode));
                }
                ShouldAction::Rerender(draw_option) => {
                    self.draw_option = draw_option;
                    return true;
                }
                ShouldAction::AddFigure(figure) => {
                    ctx.props()
                        .handler
                        .emit(ChildRequestType::AddFigure(figure));
                }
                ShouldAction::NotifyMousePositionChanged(queue) => {
                    ctx.props()
                        .handler
                        .emit(ChildRequestType::NotifyMousePositionChanged(queue));
                }
            }
        }
        false
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let mousedown = ctx.link().callback(DrawAreaMessage::MouseDown);
        let mousemove = ctx.link().callback(DrawAreaMessage::MouseMove);
        let mouseup = ctx.link().callback(DrawAreaMessage::MouseUp);
        let wheel = ctx.link().callback(DrawAreaMessage::Wheel);
        let node_ref_clone = self.data.node_ref();
        let current_mode = ctx.props().current_mode;

        html! (
            <div style="width:100%; height:100%; overflow: hidden;">
                <canvas style={canvas_css(self, current_mode)}
                    onmousedown={mousedown}
                    onmousemove={mousemove}
                    onmouseup={mouseup}
                    onwheel={wheel}
                    ref={node_ref_clone}
                />
            </div>
        )
    }
}

impl DrawArea {
    fn request_animation_frame(f: &Closure<dyn FnMut()>) -> i32 {
        web_sys::window()
            .unwrap()
            .request_animation_frame(f.as_ref().unchecked_ref())
            .unwrap()
    }

    fn render_2d_context(
        &mut self,
        canvas: HtmlCanvasElement,
        context: CanvasRenderingContext2d,
        props: &DrawAreaProps,
    ) {
        if self.draw_option == DrawOption::Remain {
            return;
        }

        canvas.set_width(canvas.client_width() as u32);
        canvas.set_height(canvas.client_height() as u32);

        let preview = Rc::new(RefCell::new(self.data.clone_preview()));
        let coordinates = self.data.coordinates().clone();
        let figure_list = props.figures.list();
        let user_list = props.shared_users.list();

        let callback = Rc::new(RefCell::new(None));
        let animation_handle_clone = self.animation_handle.clone();

        let callback_clone = callback.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            context.clear_rect(
                0.0,
                0.0,
                canvas.client_width() as f64,
                canvas.client_height() as f64,
            );

            let drawer = Drawer::new(&context, &coordinates);

            let mut list_borrow_mut = figure_list.borrow_mut();

            for figure in list_borrow_mut.iter_mut() {
                figure.accept(&drawer);
            }

            let preview_tmp = preview.borrow_mut().take();

            if let Some(mut preview_tmp) = preview_tmp {
                preview_tmp.accept(&drawer);
                *preview.borrow_mut() = Some(preview_tmp);
            }

            let mut shared_users_borrow_mut = user_list.borrow_mut();

            let mut mouse_position_all_empty = true;
            for user in shared_users_borrow_mut.iter_mut() {
                if !user.is_it_me() {
                    user.draw_mouse_cursor(&context, &coordinates);
                    if !user.check_mouse_position_queue_empty() {
                        mouse_position_all_empty = false;
                    }
                }
            }

            if mouse_position_all_empty {
                return;
            }

            let handle =
                DrawArea::request_animation_frame(callback_clone.borrow().as_ref().unwrap());
            *animation_handle_clone.borrow_mut() = Some(handle);
        });

        *callback.borrow_mut() = Some(closure);

        *self.animation_handle.borrow_mut() = Some(DrawArea::request_animation_frame(
            callback.borrow().as_ref().unwrap(),
        ));
    }

    #[allow(dead_code)]
    fn render_gl(&mut self, gl: GL, canvas: HtmlCanvasElement) {
        canvas.set_width(canvas.client_width() as u32);
        canvas.set_height(canvas.client_height() as u32);

        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        gl.clear_color(209.0 / 255.0, 209.0 / 255.0, 209.0 / 255.0, 1.0);
        gl.clear(GL::COLOR_BUFFER_BIT);

        if self.webgl_data.is_none() {
            self.webgl_data = Some(WebGLData::new(&gl).unwrap());
        }

        let rgba = Color::new(255, 0, 0, 255);

        let (start_x, start_y) = convert_figure_to_webgl(
            self.data.coordinates(),
            canvas.client_width() as f64,
            canvas.client_height() as f64,
            -100.0,
            -100.0,
        );
        let (end_x, end_y) = convert_figure_to_webgl(
            self.data.coordinates(),
            canvas.client_width() as f64,
            canvas.client_height() as f64,
            0.0,
            0.0,
        );

        let shader_program = self.webgl_data.as_ref().unwrap().shader_program();

        let drawer = DrawerGL::new(&gl, shader_program);

        let mut line = Line::new(start_x, start_y, end_x, end_y, rgba);

        line.accept(&drawer);
    }
}

#[allow(clippy::too_many_arguments)]
fn _draw_triangle(
    gl: &GL,
    shader_program: &WebGlProgram,
    x: f32,
    y: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    rgba: &Color,
) {
    let vectices: Vec<f32> = vec![x, y, x2, y2, x3, y3];
    let verts = js_sys::Float32Array::from(vectices.as_slice());
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &verts, GL::STATIC_DRAW);

    let color = gl.get_uniform_location(shader_program, "color");
    gl.uniform4f(
        color.as_ref(),
        rgba.r as f32 / 255.0,
        rgba.g as f32 / 255.0,
        rgba.b as f32 / 255.0,
        rgba.a as f32 / 255.0,
    );

    gl.draw_arrays(GL::TRIANGLES, 0, 3);
}

fn canvas_css(draw_area: &DrawArea, current_mode: DrawModeType) -> &'static str {
    if draw_area.pan_mode.is_some() {
        return "width:100%; height:100%; cursor: grabbing;";
    }

    match current_mode {
        DrawModeType::SelectMode => {
            "width:100%; height:100%; cursor: url(\"/img/cursor.png\"), auto;"
        }
        DrawModeType::LineMode => "width:100%; height:100%; cursor: crosshair;",
    }
}
