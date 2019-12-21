use termion;
use termion::color;

#[derive(Clone)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    LightBlack,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    LightWhite,
}

pub fn color_str(color: &Color, text: &String) -> String {
    match color {
        Color::Black => format!("{}", text),
        Color::Red => format!(
            "{}{}{}",
            color::Bg(color::Red),
            text,
            color::Bg(color::Black)
        ),
        Color::Green => format!(
            "{}{}{}",
            color::Bg(color::Green),
            text,
            color::Bg(color::Black)
        ),
        Color::Yellow => format!(
            "{}{}{}",
            color::Bg(color::Yellow),
            text,
            color::Bg(color::Black)
        ),
        Color::Blue => format!(
            "{}{}{}",
            color::Bg(color::Blue),
            text,
            color::Bg(color::Black)
        ),
        Color::Magenta => format!(
            "{}{}{}",
            color::Bg(color::Magenta),
            text,
            color::Bg(color::Black)
        ),
        Color::Cyan => format!(
            "{}{}{}",
            color::Bg(color::Cyan),
            text,
            color::Bg(color::Black)
        ),
        Color::White => format!(
            "{}{}{}",
            color::Bg(color::White),
            text,
            color::Bg(color::Black)
        ),
        Color::LightBlack => format!(
            "{}{}{}",
            color::Bg(color::LightBlack),
            text,
            color::Bg(color::Black)
        ),
        Color::LightRed => format!(
            "{}{}{}",
            color::Bg(color::LightRed),
            text,
            color::Bg(color::Black)
        ),
        Color::LightGreen => format!(
            "{}{}{}",
            color::Bg(color::LightGreen),
            text,
            color::Bg(color::Black)
        ),
        Color::LightYellow => format!(
            "{}{}{}",
            color::Bg(color::LightYellow),
            text,
            color::Bg(color::Black)
        ),
        Color::LightBlue => format!(
            "{}{}{}",
            color::Bg(color::LightBlue),
            text,
            color::Bg(color::Black)
        ),
        Color::LightMagenta => format!(
            "{}{}{}",
            color::Bg(color::LightMagenta),
            text,
            color::Bg(color::Black)
        ),
        Color::LightCyan => format!(
            "{}{}{}",
            color::Bg(color::LightCyan),
            text,
            color::Bg(color::Black)
        ),
        Color::LightWhite => format!(
            "{}{}{}",
            color::Bg(color::LightWhite),
            text,
            color::Bg(color::Black)
        ),
    }
}
