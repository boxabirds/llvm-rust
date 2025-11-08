//! LLVM IR Parser
//!
//! This module provides functionality to parse LLVM IR from text format (.ll files).

use std::collections::HashMap;
use crate::module::Module;
use crate::function::Function;
use crate::basic_block::BasicBlock;
use crate::instruction::{Instruction, Opcode};
use crate::value::Value;
use crate::types::Type;
use crate::context::Context;

/// Parse errors
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Unexpected token
    UnexpectedToken { expected: String, found: String, line: usize },
    /// Invalid syntax
    InvalidSyntax { message: String, line: usize },
    /// Unknown type
    UnknownType { type_name: String, line: usize },
    /// Unknown instruction
    UnknownInstruction { opcode: String, line: usize },
    /// End of file
    UnexpectedEOF,
}

/// Parse result
pub type ParseResult<T> = Result<T, ParseError>;

/// LLVM IR Parser
pub struct Parser {
    context: Context,
    tokens: Vec<Token>,
    current: usize,
    line: usize,
}

/// Token types
#[derive(Debug, Clone, PartialEq)]
enum Token {
    // Keywords
    Define,
    Declare,
    Global,
    Constant,
    Ret,
    Br,
    Label,

    // Literals
    Identifier(String),
    LocalIdent(String),
    GlobalIdent(String),
    Integer(i64),
    Float(f64),
    String(String),

    // Types
    TypeName(String),

    // Symbols
    Equal,
    Comma,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Colon,

    // Special
    EOF,
}

impl Parser {
    pub fn new(context: Context) -> Self {
        Self {
            context,
            tokens: Vec::new(),
            current: 0,
            line: 1,
        }
    }

    /// Parse a module from source code
    pub fn parse_module(&mut self, source: &str) -> ParseResult<Module> {
        self.tokenize(source)?;
        self.current = 0;

        let module = Module::new("parsed_module".to_string(), self.context.clone());

        // Parse module contents
        while !self.is_at_end() {
            if self.match_token(&Token::Define) {
                let function = self.parse_function()?;
                module.add_function(function);
            } else if self.match_token(&Token::Declare) {
                let function = self.parse_function_declaration()?;
                module.add_function(function);
            } else if self.match_token(&Token::Global) {
                // Parse global variable
                self.parse_global()?;
            } else {
                self.advance();
            }
        }

        Ok(module)
    }

    fn tokenize(&mut self, source: &str) -> ParseResult<()> {
        self.tokens.clear();
        self.line = 1;

        let mut chars = source.chars().peekable();
        let mut current_token = String::new();

        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' | '\t' | '\r' => {
                    if !current_token.is_empty() {
                        self.add_token(&current_token);
                        current_token.clear();
                    }
                    chars.next();
                }
                '\n' => {
                    if !current_token.is_empty() {
                        self.add_token(&current_token);
                        current_token.clear();
                    }
                    self.line += 1;
                    chars.next();
                }
                ';' => {
                    // Comment - skip to end of line
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c == '\n' {
                            self.line += 1;
                            break;
                        }
                    }
                }
                '=' | ',' | '(' | ')' | '{' | '}' | ':' => {
                    if !current_token.is_empty() {
                        self.add_token(&current_token);
                        current_token.clear();
                    }
                    self.tokens.push(match ch {
                        '=' => Token::Equal,
                        ',' => Token::Comma,
                        '(' => Token::LParen,
                        ')' => Token::RParen,
                        '{' => Token::LBrace,
                        '}' => Token::RBrace,
                        ':' => Token::Colon,
                        _ => unreachable!(),
                    });
                    chars.next();
                }
                '"' => {
                    // String literal
                    chars.next();
                    let mut string_val = String::new();
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c == '"' {
                            break;
                        }
                        string_val.push(c);
                    }
                    self.tokens.push(Token::String(string_val));
                }
                _ => {
                    current_token.push(ch);
                    chars.next();
                }
            }
        }

        if !current_token.is_empty() {
            self.add_token(&current_token);
        }

        self.tokens.push(Token::EOF);
        Ok(())
    }

    fn add_token(&mut self, token_str: &str) {
        let token = match token_str {
            "define" => Token::Define,
            "declare" => Token::Declare,
            "global" => Token::Global,
            "constant" => Token::Constant,
            "ret" => Token::Ret,
            "br" => Token::Br,
            "label" => Token::Label,
            _ if token_str.starts_with('@') => Token::GlobalIdent(token_str[1..].to_string()),
            _ if token_str.starts_with('%') => Token::LocalIdent(token_str[1..].to_string()),
            _ if token_str.starts_with('i') && token_str[1..].parse::<u32>().is_ok() => {
                Token::TypeName(token_str.to_string())
            }
            _ if token_str.parse::<i64>().is_ok() => {
                Token::Integer(token_str.parse().unwrap())
            }
            _ if token_str.parse::<f64>().is_ok() => {
                Token::Float(token_str.parse().unwrap())
            }
            "void" | "float" | "double" | "label" | "metadata" => {
                Token::TypeName(token_str.to_string())
            }
            _ => Token::Identifier(token_str.to_string()),
        };
        self.tokens.push(token);
    }

    fn parse_function(&mut self) -> ParseResult<Function> {
        // Parse return type
        let _return_type = self.parse_type()?;

        // Parse function name
        let name = if let Some(Token::GlobalIdent(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err(ParseError::InvalidSyntax {
                message: "Expected function name".to_string(),
                line: self.line,
            });
        };

        // Parse parameters
        self.consume(&Token::LParen)?;
        let _params = self.parse_parameter_list()?;
        self.consume(&Token::RParen)?;

        // Create function
        let fn_type = self.context.function_type(
            self.context.void_type(),
            vec![],
            false,
        );
        let function = Function::new(name, fn_type);

        // Parse body if present
        if self.match_token(&Token::LBrace) {
            while !self.match_token(&Token::RBrace) && !self.is_at_end() {
                if let Some(bb) = self.parse_basic_block()? {
                    function.add_basic_block(bb);
                }
            }
        }

        Ok(function)
    }

    fn parse_function_declaration(&mut self) -> ParseResult<Function> {
        // Similar to parse_function but without body
        self.parse_function()
    }

    fn parse_global(&mut self) -> ParseResult<()> {
        // Parse global variable
        // Simplified implementation
        Ok(())
    }

    fn parse_basic_block(&mut self) -> ParseResult<Option<BasicBlock>> {
        // Parse label if present
        let name = if let Some(Token::LocalIdent(n)) = self.peek().cloned() {
            self.advance();
            self.consume(&Token::Colon)?;
            Some(n)
        } else if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            self.consume(&Token::Colon)?;
            Some(n)
        } else {
            None
        };

        if name.is_none() {
            return Ok(None);
        }

        let bb = BasicBlock::new(name);

        // Parse instructions until next label or end of block
        while !self.is_at_end() {
            if let Some(Token::LocalIdent(_)) = self.peek() {
                // Check if this is a label (has colon after)
                if self.peek_ahead(1) == Some(&Token::Colon) {
                    break;
                }
            }
            if let Some(Token::Identifier(_)) = self.peek() {
                if self.peek_ahead(1) == Some(&Token::Colon) {
                    break;
                }
            }
            if self.match_token(&Token::RBrace) {
                self.current -= 1; // Put back the brace
                break;
            }

            if let Some(inst) = self.parse_instruction()? {
                let is_term = inst.is_terminator();
                bb.add_instruction(inst);
                if is_term {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(Some(bb))
    }

    fn parse_instruction(&mut self) -> ParseResult<Option<Instruction>> {
        // Parse instruction
        let _result_name = if let Some(Token::LocalIdent(n)) = self.peek().cloned() {
            self.advance();
            self.consume(&Token::Equal)?;
            Some(n)
        } else {
            None
        };

        // Parse opcode
        let opcode = if let Some(Token::Ret) = self.peek() {
            self.advance();
            Opcode::Ret
        } else if let Some(Token::Br) = self.peek() {
            self.advance();
            Opcode::Br
        } else if let Some(Token::Identifier(op)) = self.peek().cloned() {
            self.advance();
            self.parse_opcode(&op)?
        } else {
            return Ok(None);
        };

        // Parse operands (simplified)
        let operands = Vec::new();

        Ok(Some(Instruction::new(opcode, operands, None)))
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        if let Some(Token::TypeName(name)) = self.peek().cloned() {
            self.advance();

            let ty = match name.as_str() {
                "void" => self.context.void_type(),
                "i1" => self.context.bool_type(),
                "i8" => self.context.int8_type(),
                "i16" => self.context.int16_type(),
                "i32" => self.context.int32_type(),
                "i64" => self.context.int64_type(),
                "float" => self.context.float_type(),
                "double" => self.context.double_type(),
                "label" => self.context.label_type(),
                "metadata" => self.context.metadata_type(),
                _ if name.starts_with('i') => {
                    let bits = name[1..].parse().unwrap();
                    self.context.int_type(bits)
                }
                _ => return Err(ParseError::UnknownType {
                    type_name: name.clone(),
                    line: self.line,
                }),
            };

            Ok(ty)
        } else {
            Err(ParseError::InvalidSyntax {
                message: "Expected type".to_string(),
                line: self.line,
            })
        }
    }

    fn parse_parameter_list(&mut self) -> ParseResult<Vec<(Type, String)>> {
        let mut params = Vec::new();

        while !self.check(&Token::RParen) && !self.is_at_end() {
            let ty = self.parse_type()?;
            let name = if let Some(Token::LocalIdent(n)) = self.peek().cloned() {
                self.advance();
                n
            } else {
                format!("arg{}", params.len())
            };

            params.push((ty, name));

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(params)
    }

    fn parse_opcode(&self, opcode_str: &str) -> ParseResult<Opcode> {
        match opcode_str {
            "add" => Ok(Opcode::Add),
            "sub" => Ok(Opcode::Sub),
            "mul" => Ok(Opcode::Mul),
            "sdiv" => Ok(Opcode::SDiv),
            "udiv" => Ok(Opcode::UDiv),
            "srem" => Ok(Opcode::SRem),
            "urem" => Ok(Opcode::URem),
            "fadd" => Ok(Opcode::FAdd),
            "fsub" => Ok(Opcode::FSub),
            "fmul" => Ok(Opcode::FMul),
            "fdiv" => Ok(Opcode::FDiv),
            "frem" => Ok(Opcode::FRem),
            "and" => Ok(Opcode::And),
            "or" => Ok(Opcode::Or),
            "xor" => Ok(Opcode::Xor),
            "shl" => Ok(Opcode::Shl),
            "lshr" => Ok(Opcode::LShr),
            "ashr" => Ok(Opcode::AShr),
            "icmp" => Ok(Opcode::ICmp),
            "fcmp" => Ok(Opcode::FCmp),
            "phi" => Ok(Opcode::PHI),
            "call" => Ok(Opcode::Call),
            "alloca" => Ok(Opcode::Alloca),
            "load" => Ok(Opcode::Load),
            "store" => Ok(Opcode::Store),
            "getelementptr" => Ok(Opcode::GetElementPtr),
            "trunc" => Ok(Opcode::Trunc),
            "zext" => Ok(Opcode::ZExt),
            "sext" => Ok(Opcode::SExt),
            "fptrunc" => Ok(Opcode::FPTrunc),
            "fpext" => Ok(Opcode::FPExt),
            "fptoui" => Ok(Opcode::FPToUI),
            "fptosi" => Ok(Opcode::FPToSI),
            "uitofp" => Ok(Opcode::UIToFP),
            "sitofp" => Ok(Opcode::SIToFP),
            "ptrtoint" => Ok(Opcode::PtrToInt),
            "inttoptr" => Ok(Opcode::IntToPtr),
            "bitcast" => Ok(Opcode::BitCast),
            "select" => Ok(Opcode::Select),
            "extractvalue" => Ok(Opcode::ExtractValue),
            "insertvalue" => Ok(Opcode::InsertValue),
            "extractelement" => Ok(Opcode::ExtractElement),
            "insertelement" => Ok(Opcode::InsertElement),
            "shufflevector" => Ok(Opcode::ShuffleVector),
            _ => Err(ParseError::UnknownInstruction {
                opcode: opcode_str.to_string(),
                line: self.line,
            }),
        }
    }

    // Token manipulation helpers

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn peek_ahead(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.current + n)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Some(Token::EOF))
    }

    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(self.peek().unwrap()) == std::mem::discriminant(token)
    }

    fn consume(&mut self, token: &Token) -> ParseResult<()> {
        if self.check(token) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", token),
                found: format!("{:?}", self.peek()),
                line: self.line,
            })
        }
    }
}

/// Parse a module from a string
pub fn parse(source: &str, context: Context) -> ParseResult<Module> {
    let mut parser = Parser::new(context);
    parser.parse_module(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let ctx = Context::new();
        let source = r#"
            define void @main() {
            entry:
                ret void
            }
        "#;

        let result = parse(source, ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_empty_module() {
        let ctx = Context::new();
        let source = "";
        let result = parse(source, ctx);
        assert!(result.is_ok());
    }
}
