use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color, Shadow, Theme};

use crate::app::StatusLevel;

pub const APP_BACKGROUND: Color = Color::from_rgb(0.071, 0.078, 0.086);
pub const SURFACE: Color = Color::from_rgb(0.110, 0.122, 0.137);
pub const SURFACE_MUTED: Color = Color::from_rgb(0.153, 0.169, 0.188);
pub const VIEWPORT_BACKGROUND: Color = Color::from_rgb(0.082, 0.090, 0.102);
pub const BORDER: Color = Color::from_rgb(0.278, 0.302, 0.333);
pub const TEXT: Color = Color::from_rgb(0.902, 0.925, 0.941);
pub const TEXT_MUTED: Color = Color::from_rgb(0.651, 0.694, 0.729);
pub const ACCENT: Color = Color::from_rgb(0.176, 0.678, 0.678);
pub const ACCENT_SOFT: Color = Color::from_rgb(0.129, 0.255, 0.263);
pub const LOAD: Color = Color::from_rgb(0.957, 0.373, 0.322);
pub const SUPPORT: Color = Color::from_rgb(0.345, 0.812, 0.573);
pub const WARNING: Color = Color::from_rgb(0.929, 0.647, 0.243);

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
        button::Status::Hovered => Color::from_rgb(0.133, 0.596, 0.600),
        button::Status::Pressed => Color::from_rgb(0.098, 0.482, 0.486),
        button::Status::Disabled => Color::from_rgb(0.243, 0.263, 0.286),
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::from_rgb(0.027, 0.043, 0.047),
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
        button::Status::Pressed => Color::from_rgb(0.192, 0.212, 0.235),
        button::Status::Disabled => Color::from_rgb(0.137, 0.149, 0.165),
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
        button::Status::Hovered => Color::from_rgb(0.157, 0.310, 0.318),
        button::Status::Pressed => Color::from_rgb(0.106, 0.235, 0.243),
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

pub fn compact_input(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Active => BORDER,
        text_input::Status::Hovered => TEXT_MUTED,
        text_input::Status::Focused { .. } => ACCENT,
        text_input::Status::Disabled => SURFACE_MUTED,
    };

    text_input::Style {
        background: Background::Color(VIEWPORT_BACKGROUND),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 3.0.into(),
        },
        icon: TEXT_MUTED,
        placeholder: TEXT_MUTED,
        value: TEXT,
        selection: ACCENT_SOFT,
    }
}
