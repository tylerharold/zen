use syntect::highlighting::Style;
use termion::color;
pub fn style_to_termion(style: &Style) -> String {
    let mut escape_sequence = String::new();

    escape_sequence.push_str(&format!(
        "{}",
        color::Fg(color::Rgb(
            style.foreground.r,
            style.foreground.g,
            style.foreground.b
        ))
    ));

    escape_sequence.push_str(&format!(
        "{}",
        color::Bg(color::Rgb(
            style.background.r,
            style.background.g,
            style.background.b
        ))
    ));

    if style
        .font_style
        .contains(syntect::highlighting::FontStyle::BOLD)
    {
        escape_sequence.push_str(&format!("{}", termion::style::Bold));
    }

    if style
        .font_style
        .contains(syntect::highlighting::FontStyle::ITALIC)
    {
        escape_sequence.push_str(&format!("{}", termion::style::Italic));
    }

    escape_sequence
}
