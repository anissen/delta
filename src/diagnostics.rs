use crate::errors::Error;

#[derive(Debug, Clone)]
pub struct Diagnostics {
    errors: Vec<Error>,
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new()
    }
}

impl Diagnostics {
    pub fn new() -> Self {
        Diagnostics { errors: Vec::new() }
    }

    pub fn add_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn count(&self) -> usize {
        self.errors.len()
    }

    pub fn get_errors(&self) -> Vec<Error> {
        self.errors.clone()
    }

    pub fn to_string(&self) -> String {
        self.errors
            .iter()
            // .map(|f| format!("line {}.{}: {}\n", f.span.line, f.span.column, f.content))
            .map(|err| format!("{err:?}"))
            .collect()
    }
}
