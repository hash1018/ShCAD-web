use std::{any::Any, cell::RefCell, collections::BTreeSet, rc::Rc};

use crate::{
    algorithm::{
        math::caculate_rectangle,
        visitor::finder::{DragRectFinder, Finder},
    },
    base::{DrawOption, Rect},
    pages::workspace::{data::FigureMaintainer, draw_area::data::DrawAreaData},
};

use super::{DrawMode, ShouldAction};

enum ChangeSubMode {
    Default,
    DragSelect,
}

pub struct SelectMode {
    sub_mode: Option<Box<dyn SubSelectMode>>,
}

impl Default for SelectMode {
    fn default() -> Self {
        SelectMode::new()
    }
}

impl SelectMode {
    pub fn new() -> Self {
        SelectMode {
            sub_mode: Some(Box::new(SubSelectDefaultMode::new())),
        }
    }

    pub fn select_drag_rect(&self) -> Option<Rect> {
        if let Some(sub_mode) = self.sub_mode.as_ref() {
            if let Some(sub_select_drag_mode) =
                sub_mode.as_any().downcast_ref::<SubSelectDragMode>()
            {
                return Some(sub_select_drag_mode.select_drag_rect());
            }
        }
        None
    }

    fn change_sub_mode(&mut self, change_sub_mode: ChangeSubMode, x: f64, y: f64) {
        self.sub_mode = match change_sub_mode {
            ChangeSubMode::Default => Some(Box::new(SubSelectDefaultMode::new())),
            ChangeSubMode::DragSelect => Some(Box::new(SubSelectDragMode::new(x, y))),
        }
    }
}

impl DrawMode for SelectMode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn mouse_left_press_event(
        &mut self,
        event: web_sys::MouseEvent,
        data: &mut DrawAreaData,
        figure_maintainer: Rc<RefCell<FigureMaintainer>>,
    ) -> Option<Vec<ShouldAction>> {
        let mut sub_mode = self.sub_mode.take().unwrap();
        let (actions, change_sub_mode) =
            sub_mode.mouse_left_press_event(self, &event, data, figure_maintainer);

        if let Some(change_sub_mode) = change_sub_mode {
            let (x, y) = self.convert_figure_coordinates(&event, data);
            self.change_sub_mode(change_sub_mode, x, y);
        } else {
            self.sub_mode = Some(sub_mode);
        }

        actions
    }

    fn mouse_mouse_event(
        &mut self,
        event: web_sys::MouseEvent,
        data: &mut DrawAreaData,
        figure_maintainer: Rc<RefCell<FigureMaintainer>>,
    ) -> Option<Vec<ShouldAction>> {
        let mut sub_mode = self.sub_mode.take().unwrap();
        let (actions, change_sub_mode) =
            sub_mode.mouse_mouse_event(self, &event, data, figure_maintainer);

        if let Some(change_sub_mode) = change_sub_mode {
            let (x, y) = self.convert_figure_coordinates(&event, data);
            self.change_sub_mode(change_sub_mode, x, y);
        } else {
            self.sub_mode = Some(sub_mode);
        }

        actions
    }

    fn mouse_release_event(
        &mut self,
        event: web_sys::MouseEvent,
        data: &mut DrawAreaData,
    ) -> Option<Vec<ShouldAction>> {
        let mut sub_mode = self.sub_mode.take().unwrap();
        let (actions, change_sub_mode) = sub_mode.mouse_release_event(self, &event, data);

        if let Some(change_sub_mode) = change_sub_mode {
            let (x, y) = self.convert_figure_coordinates(&event, data);
            self.change_sub_mode(change_sub_mode, x, y);
        } else {
            self.sub_mode = Some(sub_mode);
        }

        actions
    }

    fn get_type(&self) -> super::DrawModeType {
        super::DrawModeType::SelectMode
    }
}

trait SubSelectMode {
    fn as_any(&self) -> &dyn Any;

    fn mouse_left_press_event(
        &mut self,
        select_mode: &mut SelectMode,
        event: &web_sys::MouseEvent,
        data: &mut DrawAreaData,
        figures: Rc<RefCell<FigureMaintainer>>,
    ) -> (Option<Vec<ShouldAction>>, Option<ChangeSubMode>);
    fn mouse_mouse_event(
        &mut self,
        select_mode: &mut SelectMode,
        event: &web_sys::MouseEvent,
        data: &mut DrawAreaData,
        figures: Rc<RefCell<FigureMaintainer>>,
    ) -> (Option<Vec<ShouldAction>>, Option<ChangeSubMode>);
    fn mouse_release_event(
        &mut self,
        select_mode: &mut SelectMode,
        event: &web_sys::MouseEvent,
        data: &mut DrawAreaData,
    ) -> (Option<Vec<ShouldAction>>, Option<ChangeSubMode>);
}

struct SubSelectDefaultMode {}

impl SubSelectDefaultMode {
    fn new() -> Self {
        SubSelectDefaultMode {}
    }
}

impl SubSelectMode for SubSelectDefaultMode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn mouse_left_press_event(
        &mut self,
        select_mode: &mut SelectMode,
        event: &web_sys::MouseEvent,
        data: &mut DrawAreaData,
        figure_maintainer: Rc<RefCell<FigureMaintainer>>,
    ) -> (Option<Vec<ShouldAction>>, Option<ChangeSubMode>) {
        let (x, y) = select_mode.convert_figure_coordinates(event, data);

        let finder = Finder::new((x, y), data.coordinates().zoom_rate, 6.0);

        let mut f_m_borrow_mut = figure_maintainer.borrow_mut();

        let mut actions = Vec::new();
        let mut change_sub_mode = None;

        if let Some(id) = f_m_borrow_mut.search(&finder) {
            let mut ids = BTreeSet::new();
            ids.insert(id);

            if event.shift_key() {
                if f_m_borrow_mut.check_selected(id) {
                    actions.push(ShouldAction::UpdateSelectedFigures(None, Some(ids)));
                } else {
                    actions.push(ShouldAction::SelectFigure(ids));
                }
            } else {
                let (about_to_select_set, about_unselect_set) =
                    f_m_borrow_mut.compare_selected_list(ids);
                actions.push(ShouldAction::UpdateSelectedFigures(
                    about_to_select_set,
                    about_unselect_set,
                ));
            }
        } else {
            if f_m_borrow_mut.selected_list_len() != 0 {
                actions.push(ShouldAction::UnselectFigureAll);
            }

            actions.push(ShouldAction::NotifySelectDragStart(x, y));

            change_sub_mode = Some(ChangeSubMode::DragSelect);
        }

        let actions = if actions.is_empty() {
            None
        } else {
            Some(actions)
        };

        (actions, change_sub_mode)
    }

    fn mouse_mouse_event(
        &mut self,
        _select_mode: &mut SelectMode,
        _event: &web_sys::MouseEvent,
        _data: &mut DrawAreaData,
        _figures: Rc<RefCell<FigureMaintainer>>,
    ) -> (Option<Vec<ShouldAction>>, Option<ChangeSubMode>) {
        (None, None)
    }

    fn mouse_release_event(
        &mut self,
        _select_mode: &mut SelectMode,
        _event: &web_sys::MouseEvent,
        _data: &mut DrawAreaData,
    ) -> (Option<Vec<ShouldAction>>, Option<ChangeSubMode>) {
        (None, None)
    }
}

struct SubSelectDragMode {
    prev_x: f64,
    prev_y: f64,
    current_x: f64,
    current_y: f64,
}

impl SubSelectDragMode {
    fn new(prev_x: f64, prev_y: f64) -> Self {
        SubSelectDragMode {
            prev_x,
            prev_y,
            current_x: prev_x,
            current_y: prev_y,
        }
    }

    pub fn select_drag_rect(&self) -> Rect {
        caculate_rectangle(
            (self.prev_x, self.prev_y),
            (self.current_x, self.current_y),
            false,
        )
    }
}

impl SubSelectMode for SubSelectDragMode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn mouse_left_press_event(
        &mut self,
        _select_mode: &mut SelectMode,
        _event: &web_sys::MouseEvent,
        _data: &mut DrawAreaData,
        _figure_maintainer: Rc<RefCell<FigureMaintainer>>,
    ) -> (Option<Vec<ShouldAction>>, Option<ChangeSubMode>) {
        (None, None)
    }

    fn mouse_mouse_event(
        &mut self,
        select_mode: &mut SelectMode,
        event: &web_sys::MouseEvent,
        data: &mut DrawAreaData,
        figures: Rc<RefCell<FigureMaintainer>>,
    ) -> (Option<Vec<ShouldAction>>, Option<ChangeSubMode>) {
        let (x, y) = select_mode.convert_figure_coordinates(event, data);
        self.current_x = x;
        self.current_y = y;

        let rect = caculate_rectangle(
            (self.prev_x, self.prev_y),
            (self.current_x, self.current_y),
            false,
        );
        let finder = DragRectFinder::new(rect);

        let mut actions = Vec::new();

        let len = figures.borrow().selected_list_len();
        let mut figures_borrow_mut = figures.borrow_mut();

        if let Some(set) = figures_borrow_mut.drag_search(&finder) {
            let (about_to_select_set, about_to_unselect_set) =
                figures_borrow_mut.compare_selected_list(set);
            if about_to_select_set.is_some() || about_to_unselect_set.is_some() {
                actions.push(ShouldAction::UpdateSelectedFigures(
                    about_to_select_set,
                    about_to_unselect_set,
                ));
            }
        } else if len >= 1 {
            actions.push(ShouldAction::UnselectFigureAll);
        }

        actions.push(ShouldAction::Rerender(DrawOption::DrawAll));

        (Some(actions), None)
    }

    fn mouse_release_event(
        &mut self,
        _select_mode: &mut SelectMode,
        _event: &web_sys::MouseEvent,
        _data: &mut DrawAreaData,
    ) -> (Option<Vec<ShouldAction>>, Option<ChangeSubMode>) {
        (
            Some(vec![
                ShouldAction::Rerender(DrawOption::DrawAll),
                ShouldAction::NotifySelectDragFinish,
            ]),
            Some(ChangeSubMode::Default),
        )
    }
}
