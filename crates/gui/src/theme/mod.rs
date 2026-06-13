use iced::widget::{button, container};
use iced::{Background, Border, Color, Shadow, Theme};

pub const APP_BACKGROUND: Color = Color::from_rgb(0.965, 0.973, 0.980);
pub const SURFACE: Color = Color::WHITE;
pub const SURFACE_MUTED: Color = Color::from_rgb(0.925, 0.941, 0.953);
pub const BORDER: Color = Color::from_rgb(0.765, 0.800, 0.831);
pub const TEXT: Color = Color::from_rgb(0.078, 0.118, 0.161);
pub const TEXT_MUTED: Color = Color::from_rgb(0.325, 0.384, 0.447);
pub const ACCENT: Color = Color::from_rgb(0.000, 0.369, 0.663);
pub const ACCENT_SOFT: Color = Color::from_rgb(0.850, 0.925, 0.980);
pub const DANGER: Color = Color::from_rgb(0.686, 0.110, 0.110);
pub const SUCCESS: Color = Color::from_rgb(0.000, 0.439, 0.267);

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
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn viewport(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.984, 0.988, 0.992))),
        text_color: Some(TEXT),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn status(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE)),
        text_color: Some(TEXT_MUTED),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn selected_row(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(ACCENT_SOFT)),
        text_color: Some(TEXT),
        border: Border {
            color: ACCENT,
            width: 1.0,
            radius: 4.0.into(),
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

pub fn primary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color::from_rgb(0.000, 0.298, 0.545),
        button::Status::Pressed => Color::from_rgb(0.000, 0.227, 0.420),
        button::Status::Disabled => SURFACE_MUTED,
        button::Status::Active => ACCENT,
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
        button::Status::Hovered => SURFACE_MUTED,
        button::Status::Pressed => Color::from_rgb(0.875, 0.902, 0.925),
        button::Status::Disabled => Color::from_rgb(0.945, 0.953, 0.961),
        button::Status::Active => SURFACE,
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
