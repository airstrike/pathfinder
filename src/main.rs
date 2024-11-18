use iced::widget::canvas::{self, Cache, Canvas, Event, Geometry};
use iced::widget::{
    button, center, checkbox, column, container, horizontal_space, pick_list, responsive, row,
    slider, text,
};
use iced::Alignment::Center;
use iced::{event, keyboard, mouse, time, window};
use iced::{Element, Length, Rectangle, Renderer, Subscription, Task, Theme};
use search::SearchVariant;
use std::time::Duration;

mod board;
mod pathfinder;
mod point;
mod polygon;
mod search;
mod vector;

pub use board::Board;
pub use pathfinder::{Heuristic, Pathfinder, SearchState};
pub use point::Point;
pub use polygon::{Edge, Polygon};
pub use search::Search;
pub use vector::Vector;

fn main() -> iced::Result {
    iced::application("Pathfinder", App::update, App::view)
        .window(iced::window::Settings {
            min_size: Some((800.0, 600.0).into()),
            ..Default::default()
        })
        .theme(App::theme)
        .subscription(App::subscription)
        .antialiasing(true)
        .run_with(App::new)
}

struct App {
    board_cache: Cache,
    search_cache: Cache,
    board: Board,
    is_playing: bool,
    heuristic: Heuristic,
    search: Search,
    start: Point,
    goal: Point,
    show_solution: bool,
}

impl Default for App {
    fn default() -> Self {
        let board = Board::default();
        let start = Point::new(115, 655);
        let heuristic = Heuristic::default();
        let goal = Point::new(380, 560);
        let search = Search::new(board.clone(), start, goal, heuristic);

        Self {
            board_cache: Cache::default(),
            search_cache: Cache::default(),
            heuristic,
            start,
            goal,
            search,
            board,
            is_playing: false,
            show_solution: false,
        }
    }
}

#[derive(Clone, Debug)]
enum Message {
    ToggleFullscreen,
    ChangeMode(window::Mode),

    TogglePlay,
    ToggleSolution,
    PickHeuristic(Heuristic),
    PickVariant(SearchVariant),
    SetStart(Point),
    SetGoal(Point),
    Tick,
    Back,
    Next,
    Reset,
    Finish,
    JumpTo(f32),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    fn theme(&self) -> Theme {
        Theme::TokyoNightLight
    }

    fn slide(&self) -> Element<'_, Message> {
        slider(
            0.0..=self.search.total_steps() as f32,
            self.search.current_step() as f32,
            Message::JumpTo,
        )
        .width(Length::Fill)
        .into()
    }

    fn view(&self) -> Element<Message> {
        center(
            column![
                pick_list(
                    SearchVariant::ALL,
                    Some(self.search.variant()),
                    Message::PickVariant
                ),
                responsive(move |size| {
                    center(
                        Canvas::new(self)
                            .width(Length::Fixed(size.width))
                            .height(Length::Fixed(size.height)),
                    )
                    .into()
                }),
                self.slide(),
                self.controls(),
            ]
            .align_x(Center)
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .padding(5)
        .into()
    }

    fn renew_search(&mut self, variant: SearchVariant) {
        self.search = Search::new_for_variant(
            self.board.clone(),
            self.start,
            self.goal,
            self.heuristic,
            variant,
        );
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleFullscreen => toggle_fullscreen(),
            Message::ChangeMode(mode) => {
                window::get_latest().and_then(move |id| window::change_mode(id, mode))
            }
            Message::TogglePlay => {
                self.is_playing = !self.is_playing;
                Task::none()
            }
            Message::ToggleSolution => {
                self.show_solution = !self.show_solution;
                self.search_cache.clear();
                Task::none()
            }
            Message::PickHeuristic(heuristic) => {
                self.is_playing = false;
                self.heuristic = heuristic;
                self.renew_search(self.search.variant());
                self.search_cache.clear();
                Task::none()
            }
            Message::PickVariant(variant) => {
                self.is_playing = false;
                self.renew_search(variant);
                self.search_cache.clear();
                Task::none()
            }
            Message::SetStart(start) => {
                let is_finished = self.search.is_finished();
                self.start = start;
                self.renew_search(self.search.variant());
                if is_finished {
                    self.search.jump_to(self.search.total_steps());
                }
                self.search_cache.clear();
                Task::none()
            }
            Message::SetGoal(goal) => {
                let is_finished = self.search.is_finished();
                self.goal = goal;
                self.renew_search(self.search.variant());
                if is_finished {
                    self.search.jump_to(self.search.total_steps());
                }
                self.search_cache.clear();
                Task::none()
            }
            Message::Tick => {
                if self.is_playing {
                    if !self.search.step_forward() {
                        self.is_playing = false;
                        let all_path_points = self.search.get_optimal_path().unwrap();
                        // eprintln!(
                        //     "Search finished! {}",
                        //     all_path_points
                        //         .0
                        //         .iter()
                        //         .map(|p| format!("({},{})", p.x, p.y))
                        //         .collect::<Vec<_>>()
                        //         .join(" -> ")
                        // );
                    }
                    self.search_cache.clear();
                }
                Task::none()
            }
            Message::Back => {
                self.is_playing = false;
                self.search.step_back();
                self.search_cache.clear();
                Task::none()
            }
            Message::Next => {
                self.is_playing = false;
                self.search.step_forward();
                self.search_cache.clear();
                Task::none()
            }
            Message::JumpTo(step) => {
                self.search.jump_to(step as usize);
                self.search_cache.clear();
                Task::none()
            }
            Message::Reset => {
                self.search.reset();
                self.search_cache.clear();
                Task::none()
            }
            Message::Finish => {
                self.is_playing = false;
                self.search.jump_to(self.search.total_steps());
                self.search_cache.clear();
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        use keyboard::key;

        let mut batch = vec![keyboard::on_key_press(|key, modifiers| {
            let keyboard::Key::Named(key) = key else {
                return None;
            };

            match (key, modifiers) {
                (key::Named::F11, keyboard::Modifiers::SHIFT) => Some(Message::ToggleFullscreen),
                (key::Named::Escape, _) => Some(Message::ChangeMode(window::Mode::Windowed)),
                (key::Named::Space, _) => Some(Message::TogglePlay),
                (key::Named::ArrowLeft, _) => Some(Message::Back),
                (key::Named::ArrowRight, _) => Some(Message::Next),
                (key::Named::Home, _) => Some(Message::Reset),
                (key::Named::End, _) => Some(Message::Finish),
                _ => None,
            }
        })];

        if self.is_playing {
            batch.push(time::every(Duration::from_millis(200)).map(|_| Message::Tick))
        };

        iced::Subscription::batch(batch)
    }

    fn controls<'a>(&self) -> Element<'a, Message> {
        row![
            button(text("Reset").align_x(Center))
                .style(style::reset)
                .width(Length::Fixed(100.0))
                .on_press(Message::Reset),
            button(
                text(if !self.search.is_finished() {
                    match self.is_playing {
                        true => "Pause",
                        false => {
                            if self.search.current_step() > 0 {
                                "Resume"
                            } else {
                                "Play"
                            }
                        }
                    }
                } else {
                    "Play"
                })
                .align_x(Center)
            )
            .style(style::control)
            .width(Length::Fixed(100.0))
            .on_press_maybe(if !self.search.is_finished() {
                Some(Message::TogglePlay)
            } else {
                None
            }),
            horizontal_space(),
            row![
                container(text("Heuristic:")).padding(5).align_y(Center),
                pick_list(Heuristic::ALL, Some(self.heuristic), Message::PickHeuristic)
            ],
            horizontal_space(),
            container(
                checkbox("Show Solution", self.show_solution)
                    .on_toggle(|_| { Message::ToggleSolution })
            )
            .align_y(Center)
            .padding(5),
            horizontal_space(),
            button(text("Back").align_x(Center))
                .style(style::control)
                .width(Length::Fixed(100.0))
                .on_press_maybe(if self.search.current_step() > 0 {
                    Some(Message::Back)
                } else {
                    None
                }),
            button(text("Next").align_x(Center))
                .style(style::control)
                .width(Length::Fixed(100.0))
                .on_press_maybe(if !self.search.is_finished() {
                    Some(Message::Next)
                } else {
                    None
                }),
        ]
        .spacing(5)
        .padding(5)
        .width(Length::Fill)
        .into()
    }

    // Helper function to calculate transformation parameters
    fn get_transform_params(&self, bounds: Rectangle) -> (f32, iced::Vector) {
        let (min_x, min_y, max_x, max_y) = self.board.bounds();

        let board_width = (max_x - min_x) as f32;
        let board_height = (max_y - min_y) as f32;

        // Calculate the scaling to center board within frame and its new size
        let scaling: f32 = 0.8 * (bounds.width / board_width).min(bounds.height / board_height);
        let scaled_width = board_width * scaling;
        let scaled_height = board_height * scaling;

        // Calculate translation to center the scaled board within the frame
        let translation = iced::Vector::new(
            (bounds.width - scaled_width) / 2.0 - (min_x as f32 * scaling),
            (bounds.height - scaled_height) / 2.0 + (max_y as f32 * scaling),
        );

        (scaling, translation)
    }

    // Helper function to transform screen coordinates to board coordinates
    fn screen_to_board_coords(&self, screen_pos: iced::Point, bounds: Rectangle) -> Point {
        let (scaling, translation) = self.get_transform_params(bounds);

        let board_x = (screen_pos.x - translation.x) / scaling;

        // Since the board already flips y coordinates when drawing,
        // we need to work with that convention
        let board_y = -(screen_pos.y - translation.y) / scaling;

        Point::new(board_x as i32, board_y as i32)
    }
}

impl canvas::Program<Message> for App {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let (scaling, translation) = self.get_transform_params(bounds);

        let board = self.board_cache.draw(renderer, bounds.size(), |frame| {
            frame.translate(translation);
            frame.scale(scaling);
            self.board.draw(frame);
        });

        let search = self.search_cache.draw(renderer, bounds.size(), |frame| {
            frame.translate(translation);
            frame.scale(scaling);
            self.search.draw(frame, self.show_solution);
        });

        vec![board, search]
    }

    fn update(
        &self,
        _interaction: &mut (),
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (event::Status, Option<Message>) {
        let Some(cursor_position) = cursor.position_in(bounds) else {
            return (event::Status::Ignored, None);
        };

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(button) => {
                    let message = match button {
                        mouse::Button::Left => {
                            let new_start = self.screen_to_board_coords(cursor_position, bounds);
                            Some(Message::SetStart(new_start))
                        }
                        mouse::Button::Right => {
                            let new_goal = self.screen_to_board_coords(cursor_position, bounds);
                            Some(Message::SetGoal(new_goal))
                        }
                        _ => None,
                    };

                    (event::Status::Captured, message)
                }
                _ => (event::Status::Ignored, None),
            },
            _ => (event::Status::Ignored, None),
        }
    }
}

fn toggle_fullscreen() -> Task<Message> {
    window::get_latest()
        .and_then(move |id| window::get_mode(id).map(move |mode| (id, mode)))
        .then(|(id, current_mode)| match current_mode {
            window::Mode::Fullscreen => window::change_mode(id, window::Mode::Windowed),
            _ => window::change_mode(id, window::Mode::Fullscreen),
        })
}

mod style {
    use iced::widget::button;
    use iced::Border;

    pub(super) fn control(theme: &iced::Theme, status: button::Status) -> button::Style {
        let colors = theme.extended_palette();
        let active = button::Style {
            background: Some(colors.primary.base.color.into()),
            text_color: colors.primary.base.text,
            border: Border {
                radius: 5.0.into(),
                ..Default::default()
            },
            ..Default::default()
        };

        let hovered = button::Style {
            background: Some(colors.primary.strong.color.into()),
            ..active
        };

        let disabled = button::Style {
            background: Some(colors.background.strong.color.into()),
            text_color: colors.background.base.text,
            ..active
        };

        match status {
            button::Status::Pressed => active,
            button::Status::Hovered => hovered,
            button::Status::Disabled => disabled,
            _ => active,
        }
    }

    pub(super) fn reset(theme: &iced::Theme, status: button::Status) -> button::Style {
        let colors = theme.extended_palette();
        let active = button::Style {
            background: Some(colors.danger.base.color.into()),
            text_color: colors.danger.base.text,
            border: Border {
                radius: 5.0.into(),
                ..Default::default()
            },
            ..Default::default()
        };

        let hovered = button::Style {
            background: Some(colors.danger.strong.color.into()),
            ..active
        };

        match status {
            button::Status::Pressed => active,
            button::Status::Hovered => hovered,
            _ => active,
        }
    }
}
