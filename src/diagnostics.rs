use crate::tokens::Span;

#[derive(Debug, Clone)]
pub struct Message {
    pub content: String,
    pub span: Span,
}

impl Message {
    pub fn new(content: String, span: Span) -> Self {
        Message { content, span }
    }
    pub fn from_error(content: String) -> Self {
        Message {
            content,
            span: Span { line: 0, column: 0 },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostics {
    errors: Vec<Message>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Diagnostics { errors: Vec::new() }
    }

    pub fn add_error(&mut self, message: Message) {
        self.errors.push(message);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn count(&self) -> usize {
        self.errors.len()
    }

    pub fn get_errors(&self) -> Vec<Message> {
        self.errors.clone()
    }

    pub fn to_string(&self) -> String {
        self.errors
            .iter()
            .map(|f| format!("line {}: {}", f.span.line, f.content))
            .collect()
    }
}
