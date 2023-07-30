use js_sys::Math;
use lib::{common::Color, figure::Figure};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, VecDeque},
    rc::Rc,
};
use web_sys::CanvasRenderingContext2d;

use crate::{
    algorithm::{
        coordinates_converter::convert_figure_to_device,
        visitor::{
            drawer::{Drawer, SelectedDrawer},
            finder::Finder,
        },
    },
    Coordinates,
};

#[derive(Default)]
pub struct FigureMaintainer {
    default_list: BTreeMap<usize, Box<dyn Figure>>,
    selected_list: BTreeSet<usize>,
    preview: Option<Box<dyn Figure>>,
}

impl PartialEq for FigureMaintainer {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl FigureMaintainer {
    pub fn new() -> FigureMaintainer {
        FigureMaintainer {
            default_list: BTreeMap::new(),
            selected_list: BTreeSet::new(),
            preview: None,
        }
    }

    pub fn insert_to_default(&mut self, id: usize, figure: Box<dyn Figure>) {
        self.default_list.insert(id, figure);
    }

    pub fn append_to_default(&mut self, mut figures: BTreeMap<usize, Box<dyn Figure>>) {
        self.default_list.append(&mut figures);
    }

    pub fn set_preview(&mut self, preview: Option<Box<dyn Figure>>) {
        self.preview = preview;
    }

    pub fn take_preview(&mut self) -> Option<Box<dyn Figure>> {
        self.preview.take()
    }

    pub fn clone_preview(&self) -> Option<Box<dyn Figure>> {
        self.preview.clone()
    }

    pub fn draw_default(&mut self, context: &CanvasRenderingContext2d, coordinates: &Coordinates) {
        let drawer = Drawer::new(context, coordinates);

        for (_, figure) in self.default_list.iter_mut() {
            figure.accept(&drawer);
        }

        if let Some(mut preview_tmp) = self.preview.take() {
            preview_tmp.accept(&drawer);
            self.preview = Some(preview_tmp);
        }
    }

    pub fn draw_selected(&mut self, context: &CanvasRenderingContext2d, coordinates: &Coordinates) {
        let drawer: SelectedDrawer<'_> = SelectedDrawer::new(context, coordinates);

        for id in self.selected_list.iter() {
            if let Some(figure) = self.default_list.get_mut(id) {
                figure.accept(&drawer);
            }
        }
    }

    pub fn search(&mut self, finder: &Finder) -> Option<usize> {
        for (id, figure) in self.default_list.iter_mut() {
            figure.accept(finder);
            if finder.found() {
                return Some(*id);
            }
        }

        None
    }

    pub fn select(&mut self, mut ids: BTreeSet<usize>) {
        self.selected_list.append(&mut ids);
    }

    pub fn unselect(&mut self, ids: BTreeSet<usize>) {
        for id in ids.iter() {
            self.selected_list.remove(id);
        }
    }

    pub fn unselect_all(&mut self) {
        self.selected_list.clear();
    }

    pub fn selected_list_len(&self) -> usize {
        self.selected_list.len()
    }

    pub fn check_selected(&self, id: usize) -> bool {
        self.selected_list.get(&id).is_some()
    }
}

#[derive(Default)]
pub struct SharedUsers {
    list: Rc<RefCell<Vec<SharedUser>>>,
    personal_color_generator: RefCell<PersonalColorGenerator>,
}

impl PartialEq for SharedUsers {
    fn eq(&self, other: &Self) -> bool {
        self.list.borrow().len() == other.list.borrow().len()
    }
}

impl SharedUsers {
    pub fn new() -> Self {
        Self {
            list: Rc::new(RefCell::new(Vec::new())),
            personal_color_generator: RefCell::new(PersonalColorGenerator::new()),
        }
    }

    pub fn push(&self, mut user: SharedUser) {
        let colors = self.personal_color_generator.borrow_mut().generate(1);
        user.set_color(*colors.get(0).unwrap());
        self.list.borrow_mut().push(user);
    }

    pub fn append(&self, mut users: Vec<SharedUser>) {
        let colors = self
            .personal_color_generator
            .borrow_mut()
            .generate(users.len());

        for (color, users) in colors.iter().zip(users.iter_mut()) {
            users.set_color(*color);
        }

        self.list.borrow_mut().append(&mut users);
    }

    pub fn remove(&self, user_id: String) {
        let position = self
            .list
            .borrow()
            .iter()
            .position(|user| user.user_id == user_id);
        if let Some(position) = position {
            let color = self.list.borrow().get(position).unwrap().color;
            if let Some(color) = color {
                self.personal_color_generator.borrow_mut().remove(color);
            }
            self.list.borrow_mut().remove(position);
        }
    }

    pub fn list(&self) -> Rc<RefCell<Vec<SharedUser>>> {
        self.list.clone()
    }

    pub fn update_mouse_position(&self, user_id: String, mouse_position: VecDeque<(f64, f64)>) {
        let position = self
            .list
            .borrow_mut()
            .iter()
            .position(|user| user.user_id == user_id);
        if let Some(position) = position {
            let mut list_borrow_mut = self.list.borrow_mut();
            let user = list_borrow_mut.get_mut(position).unwrap();
            user.set_mouse_position_queue(mouse_position);
        }
    }

    pub fn clear_mouse_position_queue(&self) {
        for user in self.list.borrow_mut().iter_mut() {
            user.clear_mouse_position_queue();
        }
    }
}

#[derive(Default, Debug)]
pub struct SharedUser {
    user_id: String,
    is_me: bool,
    color: Option<Color>,
    last_mouse_position: Option<(f64, f64)>,
    mouse_position_queue: VecDeque<(f64, f64)>,
    text_info: Option<(f64, f64, Color)>,
}

impl SharedUser {
    pub fn new(user_id: String, is_me: bool) -> Self {
        Self {
            user_id,
            is_me,
            color: None,
            last_mouse_position: None,
            mouse_position_queue: VecDeque::new(),
            text_info: None,
        }
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn is_it_me(&self) -> bool {
        self.is_me
    }

    fn set_color(&mut self, color: Color) {
        self.color = Some(color);
    }

    fn set_mouse_position_queue(&mut self, mut mouse_position_queue: VecDeque<(f64, f64)>) {
        self.mouse_position_queue.append(&mut mouse_position_queue);
        let len = self.mouse_position_queue.len();
        if len > 13 {
            let last = self.mouse_position_queue.pop_back().unwrap();
            let len = len - 1;
            let diff = len - 12;
            let step = len / diff;

            let mut indexes = Vec::new();

            for i in 0..diff {
                let index = len - (i + 1) * step;
                indexes.push(index);
            }

            let mut new_queue = VecDeque::new();
            for (i, p) in self.mouse_position_queue.iter().enumerate() {
                if !indexes.contains(&i) {
                    new_queue.push_back(*p);
                }
            }
            new_queue.push_back(last);

            self.mouse_position_queue = new_queue;
        }
    }

    pub fn draw_mouse_cursor(
        &mut self,
        context: &CanvasRenderingContext2d,
        coordinates: &Coordinates,
    ) {
        if let Some(mouse_position) = self.mouse_position_queue.pop_front() {
            let (x, y) = convert_figure_to_device(coordinates, mouse_position.0, mouse_position.1);
            draw_cursor_shape(
                context,
                x,
                y,
                self.color.as_ref(),
                &self.user_id,
                &mut self.text_info,
            );
            self.last_mouse_position = Some(mouse_position);
        } else if let Some(mouse_position) = self.last_mouse_position.as_ref() {
            let (x, y) = convert_figure_to_device(coordinates, mouse_position.0, mouse_position.1);
            draw_cursor_shape(
                context,
                x,
                y,
                self.color.as_ref(),
                &self.user_id,
                &mut self.text_info,
            );
        }
    }

    pub fn check_mouse_position_queue_empty(&self) -> bool {
        self.mouse_position_queue.is_empty()
    }

    pub fn clear_mouse_position_queue(&mut self) {
        if let Some(mouse_position) = self.mouse_position_queue.pop_back() {
            self.last_mouse_position = Some(mouse_position);
            self.mouse_position_queue.clear();
        }
    }
}

#[derive(Default)]
struct PersonalColorGenerator {
    colors: Vec<Color>,
}

impl PersonalColorGenerator {
    fn new() -> Self {
        Self { colors: Vec::new() }
    }

    fn generate(&mut self, count: usize) -> Vec<Color> {
        let mut vec = Vec::new();

        loop {
            if vec.len() == count {
                break;
            }

            let r = random();
            let g = random();
            let b = random();

            let color = Color::new(r, g, b, 255);

            if !self.colors.iter().any(|c| *c == color) {
                self.colors.push(color);
                vec.push(color);
            }
        }

        vec
    }

    fn remove(&mut self, color: Color) {
        if let Some(position) = self.colors.iter().position(|c| *c == color) {
            self.colors.remove(position);
        }
    }
}

fn random() -> u8 {
    //0 ~ 255
    (Math::random() * 256.0) as u8
}

fn draw_cursor_shape(
    context: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    color: Option<&Color>,
    id: &str,
    text_info: &mut Option<(f64, f64, Color)>,
) {
    if let Some(color) = color {
        let color_text = format!("rgb({0},{1},{2})", color.r, color.g, color.b);
        context.set_fill_style(&color_text.into());

        context.begin_path();
        context.move_to(x, y);
        context.line_to(x + 7.0, y + 19.0);
        context.line_to(x + 10.0, y + 10.0);
        context.line_to(x + 19.0, y + 7.0);
        context.close_path();
        context.fill();

        context.set_font("12px malgun gothic");
        context.set_text_baseline("center");

        let (width, height, color) = if let Some((width, height, color)) = text_info {
            (*width, *height, *color)
        } else {
            let metrics = context.measure_text(id).unwrap();
            let width = metrics.width();
            let height = metrics.font_bounding_box_ascent() + metrics.font_bounding_box_descent();

            let color = pick_text_color_based_on_background(color);

            *text_info = Some((width, height, color));

            (
                text_info.as_ref().unwrap().0,
                text_info.as_ref().unwrap().1,
                text_info.as_ref().unwrap().2,
            )
        };

        draw_rounded_rect(context, x - 6.0, y + 24.0, width + 12.0, height + 8.0);

        let color = format!("rgb({0},{1},{2})", color.r, color.g, color.b);
        context.set_fill_style(&color.into());
        context.fill_text(id, x, y + 24.0 + height).unwrap();
    }
}

fn pick_text_color_based_on_background(background: &Color) -> Color {
    let luminance = (0.299 * f64::from(background.r)
        + 0.587 * f64::from(background.g)
        + 0.114 * f64::from(background.b)) as u8;

    if luminance > 128 {
        Color::new(0, 0, 0, 255)
    } else {
        Color::new(255, 255, 255, 255)
    }
}

fn draw_rounded_rect(context: &CanvasRenderingContext2d, x: f64, y: f64, width: f64, height: f64) {
    let radius = height / 2.0;
    context.begin_path();
    context.move_to(x, y + radius);
    context
        .arc_to(x, y + height, x + radius, y + height, radius)
        .unwrap();
    context
        .arc_to(
            x + width,
            y + height,
            x + width,
            y + height - radius,
            radius,
        )
        .unwrap();
    context
        .arc_to(x + width, y, x + width - radius, y, radius)
        .unwrap();
    context.arc_to(x, y, x, y + radius, radius).unwrap();
    context.close_path();
    context.fill();
}
