mod board;
mod point;
mod polygon;
mod search;
mod vector;

pub use board::Board;
use iced::Alignment::Center;
pub use point::Point;
pub use polygon::Polygon;
pub use search::Search;
pub use vector::Vector;

use iced::widget::canvas::{self, Cache, Canvas, Geometry};
use iced::widget::{center, column, container, responsive, text};
use iced::{keyboard, mouse, window, Renderer, Theme};
use iced::{Element, Length, Rectangle, Subscription, Task};

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

#[derive(Default)]
struct App {
    cache: Cache,
}

#[derive(Clone, Debug)]
pub enum Message {
    ToggleFullscreen,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNightLight
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
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        use keyboard::key;

        keyboard::on_key_press(|key, modifiers| {
            let keyboard::Key::Named(key) = key else {
                return None;
            };

            match (key, modifiers) {
                (key::Named::F11, keyboard::Modifiers::SHIFT) => Some(Message::ToggleFullscreen),
                _ => None,
            }
        })
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
