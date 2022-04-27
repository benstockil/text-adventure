pub struct TextEvent<'a> {
    content: String,
    fragments: Vec<TextFragment<'a>>,
}

pub enum TextFragment<'a> {
    Text(&'a str),
    Interpolate(String),
    BeginStyle(TextStyle),
    EndStyle(TextStyle),
}

pub enum TextStyle {
    Bold,
    Italics,
    Underline,
}
