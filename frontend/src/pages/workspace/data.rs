use js_sys::Math;
use lib::{common::Color, figure::Figure};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};
use web_sys::CanvasRenderingContext2d;

use crate::{algorithm::coordinates_converter::convert_figure_to_device, Coordinates};

#[derive(Default)]
pub struct FigureList {
    list: Rc<RefCell<Vec<Box<dyn Figure>>>>,
}

impl PartialEq for FigureList {
    fn eq(&self, other: &Self) -> bool {
        self.list.borrow().len() == other.list.borrow().len()
    }
}

impl FigureList {
    pub fn new() -> FigureList {
        FigureList {
            list: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn push(&self, figure: Box<dyn Figure>) {
        self.list.borrow_mut().push(figure);
    }

    pub fn append(&self, mut figures: Vec<Box<dyn Figure>>) {
        self.list.borrow_mut().append(&mut figures);
    }

    pub fn list(&self) -> Rc<RefCell<Vec<Box<dyn Figure>>>> {
        self.list.clone()
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
            user.set_mouse_position(mouse_position);
        }
    }
}

#[derive(Default, Debug)]
pub struct SharedUser {
    user_id: String,
    is_me: bool,
    color: Option<Color>,
    last_mouse_position: Option<(f64, f64)>,
    mouse_position: VecDeque<(f64, f64)>,
}

impl SharedUser {
    pub fn new(user_id: String, is_me: bool) -> Self {
        Self {
            user_id,
            is_me,
            color: None,
            last_mouse_position: None,
            mouse_position: VecDeque::new(),
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

    fn set_mouse_position(&mut self, mut mouse_position: VecDeque<(f64, f64)>) {
        self.mouse_position.append(&mut mouse_position);
        let len = self.mouse_position.len();
        if len > 13 {
            let last = self.mouse_position.pop_back().unwrap();
            let len = len - 1;
            let diff = len - 12;
            let step = len / diff;

            let mut indexes = Vec::new();

            for i in 0..diff {
                let index = len - (i + 1) * step;
                indexes.push(index);
            }

            let mut new_queue = VecDeque::new();
            for (i, p) in self.mouse_position.iter().enumerate() {
                if !indexes.contains(&i) {
                    new_queue.push_back(*p);
                }
            }
            new_queue.push_back(last);

            self.mouse_position = new_queue;
        }
    }

    pub fn draw_mouse_cursor(
        &mut self,
        context: &CanvasRenderingContext2d,
        coordinates: &Coordinates,
    ) {
        if let Some(mouse_position) = self.mouse_position.pop_front() {
            if let Some(color) = self.color.as_ref() {
                let color = format!("rgb({0},{1},{2})", color.r, color.g, color.b);
                context.set_fill_style(&color.into());
                let (x, y) =
                    convert_figure_to_device(coordinates, mouse_position.0, mouse_position.1);
                context.rect(x, y, 20.0, 20.0);
                context.fill();

                self.last_mouse_position = Some(mouse_position);
            }
        } else if let Some(mouse_position) = self.last_mouse_position.as_ref() {
            if let Some(color) = self.color.as_ref() {
                let color = format!("rgb({0},{1},{2})", color.r, color.g, color.b);
                context.set_fill_style(&color.into());
                let (x, y) =
                    convert_figure_to_device(coordinates, mouse_position.0, mouse_position.1);
                context.rect(x, y, 20.0, 20.0);
                context.fill();
            }
        }
    }

    pub fn check_mouse_position_queue_empty(&self) -> bool {
        self.mouse_position.is_empty()
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
