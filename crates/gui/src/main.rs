mod app;
mod expression;
mod panels;
mod renderer;
mod state;
mod theme;
mod viewport;

use app::{BajrangApp, Message};
use iced::{Size, Task};

fn main() -> iced::Result {
    iced::application(|| (BajrangApp::default(), Task::none()), update, view)
        .title("Bajrang Structural Analysis")
        .window_size(Size::new(1440.0, 900.0))
        .theme(theme)
        .subscription(subscription)
        .run()
}

fn update(app: &mut BajrangApp, message: Message) -> Task<Message> {
    app.update(message)
}

fn view(app: &BajrangApp) -> iced::Element<'_, Message> {
    app.view()
}

fn subscription(app: &BajrangApp) -> iced::Subscription<Message> {
    app.subscription()
}

fn theme(_app: &BajrangApp) -> iced::Theme {
    iced::Theme::Dark
}
