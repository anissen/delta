use crate::tokens::Span;

#[derive(Debug, Clone)]
pub struct Message<'a> {
    content: String,
    span: &'a Span,
}

impl<'a> Message<'a> {
    pub fn new(content: String, span: &'a Span) -> Self {
        Message { content, span }
    }
    pub fn from_error(content: String) -> Self {
        Message {
            content,
            span: &Span { line: 0, column: 0 },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostics<'a> {
    errors: Vec<Message<'a>>,
}

impl<'a> Diagnostics<'a> {
    pub fn new() -> Self {
        Diagnostics { errors: Vec::new() }
    }

    pub fn add_error(&mut self, message: Message<'a>) {
        self.errors.push(message);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}
