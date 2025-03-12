use crate::tokens::Span;

pub struct Message<'a> {
    content: String,
    span: &'a Span,
}

impl<'a> Message<'a> {
    pub fn new(content: String, span: &'a Span) -> Self {
        Message { content, span }
    }
}

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
}
