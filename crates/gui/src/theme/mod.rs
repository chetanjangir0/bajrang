use iced::widget::{button, container};
use iced::{Background, Border, Color, Shadow, Theme};

use crate::app::StatusLevel;

pub const APP_BACKGROUND: Color = Color::from_rgb(0.944, 0.953, 0.961);
pub const SURFACE: Color = Color::WHITE;
pub const SURFACE_MUTED: Color = Color::from_rgb(0.914, 0.933, 0.949);
pub const VIEWPORT_BACKGROUND: Color = Color::from_rgb(0.976, 0.980, 0.984);
pub const BORDER: Color = Color::from_rgb(0.718, 0.757, 0.792);
pub const TEXT: Color = Color::from_rgb(0.075, 0.094, 0.118);
pub const TEXT_MUTED: Color = Color::from_rgb(0.333, 0.384, 0.435);
pub const ACCENT: Color = Color::from_rgb(0.000, 0.337, 0.624);
pub const ACCENT_SOFT: Color = Color::from_rgb(0.839, 0.918, 0.976);
pub const LOAD: Color = Color::from_rgb(0.682, 0.100, 0.094);
pub const SUPPORT: Color = Color::from_rgb(0.000, 0.392, 0.251);
pub const WARNING: Color = Color::from_rgb(0.592, 0.329, 0.000);

pub fn status_color(level: StatusLevel) -> Color {
    match level {
        StatusLevel::Neutral => TEXT_MUTED,
        StatusLevel::Success => SUPPORT,
        StatusLevel::Warning => WARNING,
        StatusLevel::Error => LOAD,
    }
}

pub fn app_background(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(APP_BACKGROUND)),
        text_color: Some(TEXT),
        ..container::Style::default()
    }
}

pub fn panel(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE)),
        text_color: Some(TEXT),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 2.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn viewport(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(VIEWPORT_BACKGROUND)),
        text_color: Some(TEXT),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 2.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn status_bar(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE)),
        text_color: Some(TEXT_MUTED),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 2.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn neutral_row(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE)),
        text_color: Some(TEXT),
        border: Border {
            color: Color::TRANSPARENT,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn inset(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE_MUTED)),
        text_color: Some(TEXT),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn primary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Active => ACCENT,
        button::Status::Hovered => Color::from_rgb(0.000, 0.282, 0.529),
        button::Status::Pressed => Color::from_rgb(0.000, 0.224, 0.431),
        button::Status::Disabled => Color::from_rgb(0.761, 0.784, 0.808),
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::WHITE,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn secondary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Active => SURFACE,
        button::Status::Hovered => SURFACE_MUTED,
        button::Status::Pressed => Color::from_rgb(0.855, 0.878, 0.898),
        button::Status::Disabled => Color::from_rgb(0.937, 0.945, 0.953),
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT,
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn tool_button(theme: &Theme, status: button::Status) -> button::Style {
    secondary_button(theme, status)
}

pub fn tool_button_active(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Active => ACCENT_SOFT,
        button::Status::Hovered => Color::from_rgb(0.788, 0.890, 0.965),
        button::Status::Pressed => Color::from_rgb(0.733, 0.855, 0.945),
        button::Status::Disabled => SURFACE_MUTED,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT,
        border: Border {
            color: ACCENT,
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}
