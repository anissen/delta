use std::fmt::Display;

use crate::errors::Error;
use crate::errors::ErrorDescription;

#[derive(Debug, Clone)]
pub struct Diagnostics {
    errors: Vec<Error>,
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Diagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for err in &self.errors {
            writeln!(f, "{err}")?;
        }
        Ok(())
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

    pub fn print(&self, source: &str) -> Vec<String> {
        self.errors.iter().map(|err| err.print(source)).collect()
    }
}
