use crate::diagnostics::Diagnostics;
use crate::errors;
use crate::expressions::ArithmeticOperations;
use crate::expressions::BinaryOperator;
use crate::expressions::BooleanOperations;
use crate::expressions::Comparisons;
use crate::expressions::EqualityOperations;
use crate::expressions::Expr;

use crate::expressions::IsArm;
use crate::expressions::IsArmPattern;
use crate::expressions::IsGuard;
use crate::expressions::MaybeNamedType;
use crate::expressions::PropertyDeclaration;
use crate::expressions::PropertyDefinition;
use crate::expressions::StringOperations;
use crate::expressions::UnaryOperator;
use crate::expressions::ValueType;
use crate::tokens::Token;
use crate::tokens::TokenKind;
use crate::tokens::TokenKind::*;
use crate::unification;

/*
program        → declaration* EOF ;
declaration    → funDecl | varDecl | expression ;
funDecl        → "\" IDENTIFIER IDENTIFIER* block ;
varDecl        → IDENTIFIER "=" expression ;
block          → "\n" INDENTATION declaration ("\n" INDENTATION declaration)* ;
expression     → assignment ;
assignment     → IDENTIFIER "=" logic_or ;
is             → string_concat "is" NEWLINE is_arm* | string_concat ;
is_arm         → INDENT ( ( "_" | expression ) block ) ;
logic_or       → logic_and ( "or" logic_or )* ;
logic_and      → equality ( "and" logic_or )* ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
block          → NEWLINE (INDENT expression NEWLINE?)*
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary | call ;
call           → call → primary "|" call_with_first_arg | primary ;
call_with_first_arg → IDENTIFIER primary* ;
primary        → "true" | "false" | NUMBER | STRING | IDENTIFIER | "(" expression ")" ;
---
NUMBER         → DIGIT+ ( "." DIGIT+ )? ;
STRING         → "\"" <any char except "\"">* "\"" ;
IDENTIFIER     → ALPHA ( ALPHA | DIGIT )* ;
ALPHA          → "a" ... "z" | "A" ... "Z" | "_" ;
DIGIT          → "0" ... "9" ;
---
MISSING:
*/

struct Parser {
    tokens: Vec<Token>,
    current: usize,
    indentation: u8,
    component_names: Vec<Token>,
}

pub fn parse(tokens: Vec<Token>) -> Result<Expr, Diagnostics> {
    Parser::new(tokens).parse()
}

// TODO(anissen): Clean up `.unwrap` in this file
impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        let non_whitespace_tokens: Vec<Token> = tokens
            .into_iter()
            .filter(|token| !matches!(token.kind, Space))
            .collect();

        let kinds = non_whitespace_tokens
            .iter()
            .filter(|token| token.kind != NewLine && token.kind != Comment && token.kind != Tab)
            .map(|t| format!("{:?} '{}'", t.kind, t.lexeme))
            .collect::<Vec<String>>()
            .join(", ");
        println!("non-whitespace tokens: {}", kinds);

        Self {
            tokens: non_whitespace_tokens,
            current: 0,
            indentation: 0,
            component_names: Vec::new(),
        }
    }

    fn parse(&mut self) -> Result<Expr, Diagnostics> {
        let mut diagnostics = Diagnostics::new();
        let mut expressions = Vec::new();
        loop {
            match self.declaration() {
                Ok(Some(expression)) => expressions.push(expression),
                Ok(None) => (), // Should we do something here?
                Err(err) => {
                    diagnostics.add_error(errors::Error::ParseErr {
                        message: err,
                        token: self.previous(),
                    });
                    // // Advance past the problematic token to avoid infinite loop
                    if !self.is_at_end() {
                        self.advance();
                    }
                }
            }
            // if let Ok(expression) = res {
            //     expressions.push(expression);
            // } else {
            //     // synchronize
            //     // TODO(anissen): Handle error synchronization
            //     // println!("Error detected: {:?}", res);
            //     // break;
            // }
            if self.is_at_end() {
                break;
            }
        }
        if !diagnostics.has_errors() {
            Ok(Expr::Block { exprs: expressions })
        } else {
            Err(diagnostics)
        }
    }

    fn declaration(&mut self) -> Result<Option<Expr>, String> {
        self.expression()
    }

    fn component(&mut self) -> Result<Option<Expr>, String> {
        let component_name = self.consume(&Identifier)?;
        let mut properties = Vec::new();
        if self.matches(&LeftBrace) {
            while !self.matches(&RightBrace) {
                if self.is_at_end() {
                    return Err("Unterminated component definition".to_string());
                }
                let property_name = self.consume(&Identifier)?;
                if !self.matches_any(&[KeywordF32, KeywordI32, KeywordStr]) {
                    return Err("Expected property type declaration".to_string());
                }
                let property_type = match self.previous().kind {
                    TokenKind::KeywordF32 => unification::Type::Float,
                    TokenKind::KeywordI32 => unification::Type::Integer,
                    TokenKind::KeywordStr => unification::Type::String,
                    _ => return Err("Unknown property type declaration".to_string()),
                };
                let property = PropertyDefinition {
                    name: property_name,
                    type_: property_type,
                };
                properties.push(property);
                let has_comma = self.matches(&Comma);
                if !has_comma {
                    self.advance();
                    break;
                }
            }
        }
        if let Some(definition) = self
            .component_names
            .iter()
            .find(|token| component_name.lexeme == token.lexeme)
        {
            return Err(format!(
                "Component '{}' already defined at line {}",
                component_name.lexeme, definition.position.line
            ));
        }
        self.component_names.push(component_name.clone());
        Ok(Some(Expr::ComponentDefinition {
            name: component_name,
            properties,
        }))
    }

    fn create(&mut self) -> Result<Option<Expr>, String> {
        let token = self.previous();
        self.consume(&LeftBracket)?; // TODO(anissen): This ought to be part of list() ?
        if let Some(components) = self.list()? {
            Ok(Some(Expr::Create {
                token: token.clone(),
                arguments: Box::new(components),
            }))
        } else {
            Err("Expected a list of components".to_string())
        }
    }

    fn expression(&mut self) -> Result<Option<Expr>, String> {
        if self.matches(&KeywordComponent) {
            self.component()
        } else if self.matches(&KeywordCreate) {
            self.create()
        } else {
            self.assignment()
        }
    }

    // assignment → IDENTIFIER "=" logic_or
    fn assignment(&mut self) -> Result<Option<Expr>, String> {
        let expr = self.query()?;
        if expr.is_some() && self.matches(&Equal) {
            let expr = expr.unwrap();
            match expr {
                Expr::Identifier { name: _ }
                | Expr::ContextIdentifier {
                    context: _,
                    name: _,
                }
                | Expr::FieldAccess {
                    identifier: _,
                    field_name: _,
                } => {
                    let operator = self.previous();
                    let value = self.assignment()?;
                    Ok(Some(Expr::Assignment {
                        target: Box::new(expr),
                        _operator: operator,
                        expr: Box::new(value.unwrap()),
                    }))
                }
                _ => Err("Invalid assignment target".to_string()),
            }
        } else {
            Ok(expr)
        }
    }

    fn tag(&mut self) -> Result<Option<Expr>, String> {
        let name = self.previous();
        let expr = if self.check(&NewLine)
            || self.check(&KeywordIs)
            || self.check(&Pipe)
            || self.is_at_end()
        {
            None
        } else {
            self.string_concat()?
        };
        Ok(Some(Expr::Value {
            value: ValueType::Tag {
                name: name.clone(),
                payload: expr.map(Box::new),
            },
            token: name,
        }))
    }

    fn list(&mut self) -> Result<Option<Expr>, String> {
        let token = self.previous();
        let mut list_elements = Vec::new();

        loop {
            if self.is_at_end() {
                return Err("Unexpected end of input".to_string());
            }
            if self.matches(&RightBracket) {
                break;
            }
            if !list_elements.is_empty() {
                self.consume(&Comma)?;
            }
            if let Some(expr) = self.expression()? {
                list_elements.push(expr)
            }
        }

        Ok(Some(Expr::Value {
            value: ValueType::List(list_elements),
            token,
        }))
    }

    // query → ...
    fn query(&mut self) -> Result<Option<Expr>, String> {
        if self.matches(&KeywordQuery) {
            self.comment();
            self.consume(&NewLine)?;
            self.increase_indentation();
            self.consume_indentation()?;

            let mut include_components = vec![];
            let mut exclude_components = vec![];
            // parse components
            while !self.check(&NewLine) {
                if self.is_at_end() {
                    return Err("Unexpected end of input".to_string());
                }

                if !include_components.is_empty() {
                    self.consume(&Comma)?;
                }

                let should_exclude = self.matches(&KeywordNot);

                if should_exclude {
                    let type_ = self.consume(&Identifier)?;
                    exclude_components.push(type_);
                } else {
                    let type_ = self.consume(&Identifier)?;
                    let name = self.optional(&Identifier);
                    include_components.push(MaybeNamedType { type_, name });
                }
            }

            // expr for matches
            if let Some(expr) = self.block()? {
                self.decrease_indentation();
                Ok(Some(Expr::Query {
                    include_components,
                    exclude_components,
                    expr: Box::new(expr),
                }))
            } else {
                Err("Unexpected end of input".to_string())
            }
        } else {
            self.is()
        }
    }

    // is → string_concat "is" NEWLINE is_arm* | string_concat
    fn is(&mut self) -> Result<Option<Expr>, String> {
        let expr = self.string_concat()?;
        if self.matches(&KeywordIs) {
            self.comment();
            self.consume(&NewLine)?;
            self.increase_indentation();
            let mut arms = vec![];
            let mut has_default = false;
            while self.matches_indentation() {
                self.consume_indentation()?;
                if self.matches(&Comment) {
                    self.consume(&NewLine)?;
                    continue;
                }
                let arm = self.is_arm()?;
                if has_default {
                    return match arm.pattern {
                        IsArmPattern::Default => {
                            Err("An `is` block cannot have multiple default arms.".to_string())
                        }
                        _ => Err("Unreachable due to default arm above.".to_string()),
                    };
                }
                has_default = matches!(arm.pattern, IsArmPattern::Default) && arm.guard.is_none();

                // TODO(anissen): Check for multiple capture arms or arms after a capture arm

                arms.push(arm);
            }
            if arms.is_empty() {
                return Err("`is` block must have at least one arm".to_string());
            }
            self.decrease_indentation();
            Ok(Some(Expr::Is {
                expr: Box::new(expr.unwrap()),
                arms,
            }))
        } else {
            Ok(expr)
        }
    }

    // is_arm → INDENT ( ( "_" | expression ) block )
    fn is_arm(&mut self) -> Result<IsArm, String> {
        let pattern = if self.matches(&Underscore) {
            IsArmPattern::Default
        } else if let Some(pattern) = self.expression()? {
            match pattern {
                Expr::Identifier { name } => IsArmPattern::Capture { identifier: name },
                Expr::Value {
                    value:
                        ValueType::Tag {
                            name: tag_name,
                            payload,
                        },
                    token,
                } => {
                    if let Some(payload) = payload {
                        match *payload {
                            Expr::Identifier { name } => IsArmPattern::CaptureTagPayload {
                                tag_name,
                                identifier: name,
                            },
                            expr => IsArmPattern::Expression(Expr::Value {
                                value: ValueType::Tag {
                                    name: tag_name,
                                    payload: Some(Box::new(expr)),
                                },
                                token,
                            }),
                        }
                    } else {
                        // Simple tag
                        IsArmPattern::Expression(Expr::Value {
                            value: ValueType::Tag {
                                name: tag_name,
                                payload: None,
                            },
                            token,
                        })
                    }
                }
                _ => IsArmPattern::Expression(pattern),
            }
        } else {
            return Err("Error parsing pattern of `is` arm".to_string());
        };

        let guard = if self.matches(&KeywordIf) {
            if let Some(condition) = self.expression()? {
                Some(IsGuard {
                    token: self.previous(),
                    condition,
                })
            } else {
                return Err("Error parsing if-guard of `is` arm".to_string());
            }
        } else {
            None
        };

        self.comment();

        if let Some(block) = self.block()? {
            Ok(IsArm {
                pattern,
                guard,
                block,
            })
        } else {
            Err("Error parsing block of `is` arm".to_string())
        }
    }

    // string_concat → STRING "{" logic_or "}";
    fn string_concat(&mut self) -> Result<Option<Expr>, String> {
        let mut expr = self.logic_or()?;
        while expr.is_some() && self.matches(&StringConcat) {
            let token = self.previous();
            let right = self.logic_or()?;
            expr = Some(Expr::Binary {
                left: Box::new(expr.unwrap()),
                operator: BinaryOperator::StringOperation(StringOperations::StringConcat),
                token,
                right: Box::new(right.unwrap()),
            });
        }
        Ok(expr)
    }

    // logic_or → logic_and ( "or" logic_or )* ;
    fn logic_or(&mut self) -> Result<Option<Expr>, String> {
        let expr = self.logic_and()?;
        if self.matches(&KeywordOr) {
            let token = self.previous();
            let right = self.logic_or()?;
            Ok(Some(Expr::Binary {
                left: Box::new(expr.unwrap()),
                operator: BinaryOperator::BooleanOperation(BooleanOperations::Or),
                token,
                right: Box::new(right.unwrap()),
            }))
        } else {
            Ok(expr)
        }
    }

    // logic_and → equality ( "and" logic_or )* ;
    fn logic_and(&mut self) -> Result<Option<Expr>, String> {
        let expr = self.equality()?;
        if self.matches(&KeywordAnd) {
            let token = self.previous();
            let right = self.logic_or()?;
            Ok(Some(Expr::Binary {
                left: Box::new(expr.unwrap()),
                operator: BinaryOperator::BooleanOperation(BooleanOperations::And),
                token,
                right: Box::new(right.unwrap()),
            }))
        } else {
            Ok(expr)
        }
    }

    // equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Option<Expr>, String> {
        let expr = self.comparison()?;
        if expr.is_some() && self.matches_any(&[EqualEqual, BangEqual]) {
            let token = self.previous();
            let right = self.comparison()?;
            let operator = match token.kind {
                EqualEqual => BinaryOperator::Equality(EqualityOperations::Equal),
                BangEqual => BinaryOperator::Equality(EqualityOperations::NotEqual),

                _ => unreachable!(),
            };
            Ok(Some(Expr::Binary {
                left: Box::new(expr.unwrap()),
                operator,
                token,
                right: Box::new(right.unwrap()),
            }))
        } else {
            Ok(expr)
        }
    }

    // comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Result<Option<Expr>, String> {
        let expr = self.term()?;
        if expr.is_some()
            && self.matches_any(&[
                LeftChevron,
                LeftChevronDot,
                LeftChevronEqual,
                LeftChevronEqualDot,
                RightChevron,
                RightChevronDot,
                RightChevronEqual,
                RightChevronEqualDot,
            ])
        {
            let token = self.previous();
            let right = self.term()?;
            let operator = match token.kind {
                LeftChevron => BinaryOperator::IntegerComparison(Comparisons::LessThan),
                LeftChevronDot => BinaryOperator::FloatComparison(Comparisons::LessThan),
                LeftChevronEqual => BinaryOperator::IntegerComparison(Comparisons::LessThanEqual),
                LeftChevronEqualDot => BinaryOperator::FloatComparison(Comparisons::LessThanEqual),
                RightChevron => BinaryOperator::IntegerComparison(Comparisons::GreaterThan),
                RightChevronDot => BinaryOperator::FloatComparison(Comparisons::GreaterThan),
                RightChevronEqual => {
                    BinaryOperator::IntegerComparison(Comparisons::GreaterThanEqual)
                }
                RightChevronEqualDot => {
                    BinaryOperator::FloatComparison(Comparisons::GreaterThanEqual)
                }
                _ => unreachable!(),
            };
            Ok(Some(Expr::Binary {
                left: Box::new(expr.unwrap()),
                token,
                operator,
                right: Box::new(right.unwrap()),
            }))
        } else {
            Ok(expr)
        }
    }

    // term → factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Result<Option<Expr>, String> {
        let mut expr = self.factor()?;
        while expr.is_some() && self.matches_any(&[Plus, PlusDot, Minus, MinusDot]) {
            let token = self.previous();
            let operator = match token.kind {
                Plus => BinaryOperator::IntegerOperation(ArithmeticOperations::Addition),
                PlusDot => BinaryOperator::FloatOperation(ArithmeticOperations::Addition),
                Minus => BinaryOperator::IntegerOperation(ArithmeticOperations::Subtraction),
                MinusDot => BinaryOperator::FloatOperation(ArithmeticOperations::Subtraction),
                _ => unreachable!(),
            };
            let right = self.factor()?;
            expr = Some(Expr::Binary {
                left: Box::new(expr.unwrap()),
                operator,
                token,
                right: Box::new(right.unwrap()),
            });
        }
        Ok(expr)
    }

    // factor → unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<Option<Expr>, String> {
        let mut expr = self.unary()?;
        while expr.is_some()
            && self.matches_any(&[Slash, SlashDot, Star, StarDot, Percent, PercentDot])
        {
            let token = self.previous();
            let operator = match token.kind {
                Slash => BinaryOperator::IntegerOperation(ArithmeticOperations::Division),
                SlashDot => BinaryOperator::FloatOperation(ArithmeticOperations::Division),
                Star => BinaryOperator::IntegerOperation(ArithmeticOperations::Multiplication),
                StarDot => BinaryOperator::FloatOperation(ArithmeticOperations::Multiplication),
                Percent => BinaryOperator::IntegerOperation(ArithmeticOperations::Modulus),
                PercentDot => BinaryOperator::FloatOperation(ArithmeticOperations::Modulus),
                _ => unreachable!(),
            };
            let right = self.unary()?;
            expr = Some(Expr::Binary {
                left: Box::new(expr.unwrap()),
                operator,
                token,
                right: Box::new(right.unwrap()),
            });
        }
        Ok(expr)
    }

    // unary → ( "!" | "-" ) unary | call ;
    fn unary(&mut self) -> Result<Option<Expr>, String> {
        if self.matches_any(&[Bang, Minus]) {
            let token = self.previous();
            let operator = match token.kind {
                Bang => UnaryOperator::Not,
                Minus => UnaryOperator::Negation,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            Ok(Some(Expr::Unary {
                operator,
                token,
                expr: Box::new(right.unwrap()),
            }))
        } else {
            self.call()
        }
    }

    // call → primary "|" call_with_first_arg | primary
    fn call(&mut self) -> Result<Option<Expr>, String> {
        let expr = self.primary()?;
        let token = self.previous();
        if self.matches(&Pipe) {
            self.call_with_first_arg(expr.unwrap(), token)
        } else {
            Ok(expr)
        }
    }

    // call_with_first_arg → IDENTIFIER primary*
    fn call_with_first_arg(&mut self, expr: Expr, _token: Token) -> Result<Option<Expr>, String> {
        self.consume(&Identifier)?;
        let previous = self.previous();
        // TODO(anissen): Check that function name exists and is a function
        let first_arg = expr;
        let mut args = vec![first_arg];
        // TODO(anissen): Checking for string concatenation here does not feel right
        while !self.is_at_end()
            && !self.check(&NewLine)
            && !self.check(&Pipe)
            && !self.check(&StringConcat)
            && !self.check(&RightParen)
        {
            let arg = self.primary()?; // precedence after string concatenation
            if let Some(arg) = arg {
                args.push(arg);
            }
        }
        let call_expr = Expr::Call {
            name: previous,
            args,
        };
        if self.matches(&Pipe) {
            self.call_with_first_arg(call_expr, self.previous())
        } else {
            Ok(Some(call_expr))
        }
    }

    // function → IDENTIFIER* block
    fn function(&mut self) -> Result<Option<Expr>, String> {
        let slash = self.previous();
        let mut params = vec![];
        while self.matches(&Identifier) {
            let param = self.previous();
            params.push(param);
        }
        let expr = self.block()?;
        // TODO(anissen): Add function to some meta data?
        Ok(Some(Expr::Value {
            value: ValueType::Function {
                params,
                expr: Box::new(expr.unwrap()),
            },
            token: slash,
        }))
    }

    // block → NEWLINE (INDENT expression NEWLINE?)*
    fn block(&mut self) -> Result<Option<Expr>, String> {
        self.consume(&NewLine)?;
        self.increase_indentation();
        let mut exprs = vec![];
        loop {
            self.consume_indentation()?;
            if let Some(expr) = self.expression()? {
                exprs.push(expr);
            }

            self.comment();

            // Required when having nested blocks to avoid each consuming the newline
            if self.check(&NewLine) {
                self.consume(&NewLine)?;
            }

            if !self.matches_indentation() {
                // TODO(anissen): Should probably Err if indentation is wrong
                // return Err("Unexpected indentation".to_string());
                break;
            }
        }
        self.decrease_indentation();
        Ok(Some(Expr::Block { exprs }))
    }

    // whitespace → NEWLINE | COMMENT
    fn whitespace(&mut self) -> Result<Option<Expr>, String> {
        if self.matches_any(&[NewLine, Comment]) || self.is_at_end() {
            Ok(None)
        } else {
            let error = format!(
                "Parse error of kind {:?} at {:?}",
                self.peek().kind,
                self.previous().lexeme,
            );
            Err(error)
        }
    }

    // primary → "true" | "false" | INTEGER | FLOAT | STRING | IDENTIFIER | "(" expression ")" ;
    fn primary(&mut self) -> Result<Option<Expr>, String> {
        if self.matches(&Identifier) {
            let name = self.previous();
            let is_capitalized = name.lexeme.chars().take(1).any(|c| c.is_uppercase());
            if is_capitalized {
                let mut properties = Vec::new();

                // Parse potential struct initialization
                if self.matches(&LeftBrace) {
                    while !self.matches(&RightBrace) {
                        let property_name = self.consume(&Identifier)?;
                        let value = self.expression()?;

                        if self.is_at_end() {
                            return Err("Unterminated data initialization".to_string());
                        }

                        let property = PropertyDeclaration {
                            name: property_name,
                            value: value.unwrap(),
                        };
                        properties.push(property);

                        if !self.matches(&Comma) {
                            self.advance();
                            break;
                        }
                    }
                }

                // properties.sort_by(|a, b| a.name.lexeme.cmp(&b.name.lexeme));
                Ok(Some(Expr::Value {
                    value: ValueType::Component {
                        name: name.clone(),
                        properties,
                    },
                    token: name,
                }))
            } else if self.matches(&Dot) {
                // Field access
                let field_name = self.consume(&Identifier)?;
                Ok(Some(Expr::FieldAccess {
                    identifier: name,
                    field_name,
                }))
            } else {
                // Plain identifier
                Ok(Some(Expr::Identifier { name }))
            }
        } else if self.matches(&TokenKind::Context) {
            let name = self.previous();
            if self.matches(&TokenKind::Dot) {
                let identifier = self.consume(&TokenKind::Identifier)?;
                Ok(Some(Expr::ContextIdentifier {
                    context: name,
                    name: identifier,
                }))
            } else {
                Ok(Some(Expr::Context { name }))
            }
        } else if self.matches(&Integer) {
            let lexeme = self.previous().lexeme;
            let value = lexeme.parse::<i32>();
            match value {
                Ok(value) => Ok(Some(Expr::Value {
                    value: ValueType::Integer(value),
                    token: self.previous().clone(),
                })),
                Err(err) => Err(err.to_string()),
            }
        } else if self.matches(&Float) {
            let lexeme = self.previous().lexeme;
            let value = lexeme.parse::<f32>();
            match value {
                Ok(value) => Ok(Some(Expr::Value {
                    value: ValueType::Float(value),
                    token: self.previous().clone(),
                })),
                Err(err) => Err(err.to_string()),
            }
        } else if self.matches(&False) {
            Ok(Some(Expr::Value {
                value: ValueType::Boolean(false),
                token: self.previous().clone(),
            }))
        } else if self.matches(&True) {
            Ok(Some(Expr::Value {
                value: ValueType::Boolean(true),
                token: self.previous().clone(),
            }))
        } else if self.matches(&TokenKind::Text) {
            let lexeme = self.previous().lexeme;
            Ok(Some(Expr::Value {
                value: ValueType::String(lexeme),
                token: self.previous().clone(),
            }))
        } else if self.matches(&LeftParen) {
            let expr = self.expression()?;
            self.consume(&RightParen)?;
            Ok(Some(Expr::Grouping(Box::new(expr.unwrap()))))
        } else if self.matches(&BackSlash) {
            self.function()
        } else if self.matches(&Tag) {
            self.tag()
        } else if self.matches(&LeftBracket) {
            self.list()
        } else {
            self.whitespace()
        }
    }

    fn matches_indentation(&self) -> bool {
        (0..self.indentation as usize).all(|i| {
            self.tokens.len() > self.current + 1 && self.tokens[self.current + i].kind == Tab
        })
    }

    fn matches_any(&mut self, kinds: &[TokenKind]) -> bool {
        kinds.iter().any(|k| self.matches(k))
    }

    fn matches(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, kind: &TokenKind) -> Result<Token, String> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            let message = format!("Expected {kind:?} but found '{}'", self.peek().lexeme);
            Err(message.to_string())
        }
    }

    fn optional(&mut self, kind: &TokenKind) -> Option<Token> {
        if self.check(kind) {
            Some(self.advance())
        } else {
            None
        }
    }

    fn comment(&mut self) {
        self.optional(&Comment);
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().kind == kind
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn increase_indentation(&mut self) {
        self.indentation += 1;
    }

    fn decrease_indentation(&mut self) {
        self.indentation -= 1;
    }

    fn consume_indentation(&mut self) -> Result<(), String> {
        for _ in 0..self.indentation {
            self.consume(&Tab)?;
        }
        Ok(())
    }
}
