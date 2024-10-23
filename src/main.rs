use iced::widget::canvas::{self, Cache, Canvas, Geometry};
use iced::widget::{
    button, center, checkbox, column, container, horizontal_space, pick_list, responsive, row,
    slider, text,
};
use iced::Alignment::Center;
use iced::{keyboard, mouse, time, window, Renderer, Theme};
use iced::{Element, Length, Rectangle, Subscription, Task};
use std::time::Duration;

mod board;
mod point;
mod polygon;
mod search;
mod vector;

pub use board::Board;
pub use point::Point;
pub use polygon::{Edge, Polygon};
pub use search::{Heuristic, Search};
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
    cache: Cache,
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
        let start = Point::new(board.bounds().0, board.bounds().1);
        let heuristic = Heuristic::default();
        let goal = Point::new(board.bounds().2, board.bounds().3);
        let search = Search::new(board.clone(), start, goal, heuristic);

        Self {
            cache: Cache::default(),
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
pub enum Message {
    ToggleFullscreen,
    TogglePlay,
    ToggleSolution,
    PickHeuristic(Heuristic),
    SetStart(Point),
    SetGoal(Point),
    Tick,
    Back,
    Next,
    Reset,
    JumpTo(f32),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNightLight
    }

    fn slide(&self) -> Element<'_, Message> {
        slider(
            0.0..=self.search.total_steps() as f32 - 1.0,
            self.search.current_step() as f32,
            Message::JumpTo,
        )
        .width(Length::Fill)
        .into()
    }

    fn view(&self) -> Element<Message> {
        center(
            column![
                container(text("Press Shift + F11 to toggle fullscreen"))
                    .padding(5)
                    .style(container::rounded_box),
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

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleFullscreen => toggle_fullscreen(),
            Message::TogglePlay => {
                self.is_playing = !self.is_playing;
                Task::none()
            }
            Message::ToggleSolution => {
                self.show_solution = !self.show_solution;
                self.cache.clear();
                Task::none()
            }
            Message::PickHeuristic(heuristic) => {
                self.is_playing = false;
                self.heuristic = heuristic;
                self.search =
                    Search::new(self.board.clone(), self.start, self.goal, self.heuristic);
                self.cache.clear();
                Task::none()
            }
            Message::SetStart(start) => {
                self.start = start;
                self.search =
                    Search::new(self.board.clone(), self.start, self.goal, self.heuristic);
                Task::none()
            }
            Message::SetGoal(goal) => {
                self.goal = goal;
                self.search =
                    Search::new(self.board.clone(), self.start, self.goal, self.heuristic);
                Task::none()
            }
            Message::Tick => {
                if self.is_playing {
                    if !self.search.step_forward() {
                        self.is_playing = false;
                    }
                    self.cache.clear();
                }
                Task::none()
            }
            Message::Back => {
                self.is_playing = false;
                self.search.step_back();
                Task::none()
            }
            Message::Next => {
                self.is_playing = false;
                self.search.step_forward();
                self.cache.clear();
                Task::none()
            }
            Message::JumpTo(step) => {
                self.search.jump_to(step as usize);
                self.cache.clear();
                Task::none()
            }
            Message::Reset => {
                self.search.reset();
                self.cache.clear();
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
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let board = Board::default();
            let (min_x, min_y, max_x, max_y) = board.bounds();

            // Calculate the board's original bounding rectangle
            let board_width = (max_x - min_x) as f32;
            let board_height = (max_y - min_y) as f32;

            // Calculate the scaling factor to make the board fit within 80% of the frame size
            let scaling: f32 = 0.8 * (bounds.width / board_width).min(bounds.height / board_height);

            // Calculate the new size of the scaled board
            let scaled_width = board_width * scaling;
            let scaled_height = board_height * scaling;

            // Calculate translation to center the scaled board within the frame
            let translation = iced::Vector::new(
                (bounds.width - scaled_width) / 2.0 - (min_x as f32 * scaling),
                (bounds.height - scaled_height) / 2.0 - (min_y as f32 * scaling),
            );

            frame.translate(translation);
            frame.scale(scaling);
            board.draw(frame);

            self.search.draw(frame, self.show_solution);
        });

        vec![geometry]
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
