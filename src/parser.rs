//! LLVM IR Parser
//!
//! This module provides functionality to parse LLVM IR from text format (.ll files).

use crate::lexer::{Lexer, Token};
use crate::module::{Module, GlobalVariable};
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
    UnexpectedToken { expected: String, found: String, position: usize },
    /// Invalid syntax
    InvalidSyntax { message: String, position: usize },
    /// Unknown type
    UnknownType { type_name: String, position: usize },
    /// Unknown instruction
    UnknownInstruction { opcode: String, position: usize },
    /// End of file
    UnexpectedEOF,
    /// Lexer error
    LexerError(String),
}

/// Parse result
pub type ParseResult<T> = Result<T, ParseError>;

/// LLVM IR Parser
pub struct Parser {
    context: Context,
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(context: Context) -> Self {
        Self {
            context,
            tokens: Vec::new(),
            current: 0,
        }
    }

    /// Parse a module from source code
    pub fn parse_module(&mut self, source: &str) -> ParseResult<Module> {
        // Tokenize
        let mut lexer = Lexer::new(source);
        self.tokens = lexer.tokenize().map_err(ParseError::LexerError)?;
        self.current = 0;

        let module = Module::new("parsed_module".to_string(), self.context.clone());

        // Parse module contents with safety limit to prevent infinite loops
        let mut iterations = 0;
        const MAX_MODULE_ITERATIONS: usize = 100000;

        while !self.is_at_end() && iterations < MAX_MODULE_ITERATIONS {
            iterations += 1;
            // Skip metadata and attributes at module level
            if self.check(&Token::Exclaim) || self.check(&Token::Attributes) {
                self.skip_until_newline_or_semicolon();
                continue;
            }

            // Parse target datalayout/triple
            if self.match_token(&Token::Target) {
                self.parse_target_directive()?;
                continue;
            }

            // Parse source_filename
            if self.match_token(&Token::Source_filename) {
                self.parse_source_filename()?;
                continue;
            }

            // Parse type declarations
            if self.peek_global_ident().is_some() && self.peek_ahead(1) == Some(&Token::Equal)
                && self.peek_ahead(2) == Some(&Token::Type) {
                self.parse_type_declaration()?;
                continue;
            }

            // Parse global variables
            if self.peek_global_ident().is_some() && self.peek_ahead(1) == Some(&Token::Equal)
                && (self.peek_ahead(2) == Some(&Token::Global) || self.peek_ahead(2) == Some(&Token::Constant)) {
                let global = self.parse_global_variable()?;
                module.add_global(global);
                continue;
            }

            // Parse function declarations
            if self.match_token(&Token::Declare) {
                let function = self.parse_function_declaration()?;
                module.add_function(function);
                continue;
            }

            // Parse function definitions
            if self.match_token(&Token::Define) {
                let function = self.parse_function_definition()?;
                module.add_function(function);
                continue;
            }

            // Parse attribute group definitions: attributes #0 = { ... }
            if self.match_token(&Token::Attributes) {
                // Skip attribute group ID (#0, #1, etc.)
                if self.check_attr_group_id() {
                    self.advance();
                }
                // Skip '='
                self.match_token(&Token::Equal);
                // Skip attribute list in braces
                if self.match_token(&Token::LBrace) {
                    while !self.check(&Token::RBrace) && !self.is_at_end() {
                        self.advance();
                    }
                    self.match_token(&Token::RBrace);
                }
                continue;
            }

            // Skip unknown tokens
            if !self.is_at_end() {
                self.advance();
            }
        }

        if iterations >= MAX_MODULE_ITERATIONS {
            return Err(ParseError::InvalidSyntax {
                message: format!("Module parsing exceeded maximum iterations ({}), possible infinite loop", MAX_MODULE_ITERATIONS),
                position: self.current,
            });
        }

        Ok(module)
    }

    fn parse_target_directive(&mut self) -> ParseResult<()> {
        // target datalayout = "..."
        // target triple = "..."
        if self.match_token(&Token::Datalayout) || self.match_token(&Token::Triple) {
            self.consume(&Token::Equal)?;
            if let Some(Token::StringLit(_)) = self.peek() {
                self.advance();
            }
        }
        Ok(())
    }

    fn parse_source_filename(&mut self) -> ParseResult<()> {
        // source_filename = "..."
        self.consume(&Token::Equal)?;
        if let Some(Token::StringLit(_)) = self.peek() {
            self.advance();
        }
        Ok(())
    }

    fn parse_type_declaration(&mut self) -> ParseResult<()> {
        // %TypeName = type { ... }
        self.advance(); // global ident
        self.consume(&Token::Equal)?;
        self.consume(&Token::Type)?;
        self.parse_type()?;
        Ok(())
    }

    fn parse_global_variable(&mut self) -> ParseResult<GlobalVariable> {
        // @name = [linkage] [visibility] global/constant type [initializer]
        let name = self.expect_global_ident()?;
        self.consume(&Token::Equal)?;

        // Skip linkage and visibility keywords
        self.skip_linkage_and_visibility();

        let is_constant = if self.match_token(&Token::Constant) {
            true
        } else if self.match_token(&Token::Global) {
            false
        } else {
            return Err(ParseError::InvalidSyntax {
                message: "Expected 'global' or 'constant'".to_string(),
                position: self.current,
            });
        };

        let ty = self.parse_type()?;

        // Parse initializer if present
        let initializer = if !self.is_at_end() && !self.check_global_ident() && !self.check(&Token::Define) && !self.check(&Token::Declare) {
            // Skip initializer parsing for now (complex)
            None
        } else {
            None
        };

        Ok(GlobalVariable {
            name,
            ty,
            is_constant,
            initializer,
        })
    }

    fn parse_function_declaration(&mut self) -> ParseResult<Function> {
        // declare [cc] [ret attrs] [!metadata] type @name([params])
        self.skip_linkage_and_visibility(); // For calling conventions
        self.skip_attributes();

        // Skip metadata attachments before return type (e.g., declare !dbg !12 i32 @foo())
        while self.is_metadata_token() {
            self.skip_metadata();
        }

        let return_type = self.parse_type()?;
        let name = self.expect_global_ident()?;

        self.consume(&Token::LParen)?;
        let param_types = self.parse_parameter_types()?;
        self.consume(&Token::RParen)?;

        // Skip function attributes
        self.skip_function_attributes();

        let fn_type = self.context.function_type(return_type, param_types, false);
        Ok(Function::new(name, fn_type))
    }

    fn parse_function_definition(&mut self) -> ParseResult<Function> {
        // define [linkage] [ret attrs] [!metadata] type @name([params]) [fn attrs] { body }
        self.skip_linkage_and_visibility();
        self.skip_attributes();

        // Skip metadata attachments before return type
        while self.is_metadata_token() {
            self.skip_metadata();
        }

        let return_type = self.parse_type()?;
        let name = self.expect_global_ident()?;

        self.consume(&Token::LParen)?;
        let params = self.parse_parameters()?;
        self.consume(&Token::RParen)?;

        // Skip function attributes
        self.skip_function_attributes();

        // Create function
        let param_types: Vec<Type> = params.iter().map(|(ty, _)| ty.clone()).collect();
        let fn_type = self.context.function_type(return_type, param_types, false);
        let function = Function::new(name, fn_type);

        // Set arguments
        let args: Vec<Value> = params.iter().enumerate().map(|(idx, (ty, name))| {
            Value::argument(ty.clone(), idx, Some(name.clone()))
        }).collect();
        function.set_arguments(args);

        // Parse body if present with safety limit to prevent infinite loops
        if self.match_token(&Token::LBrace) {
            let mut bb_count = 0;
            const MAX_BASIC_BLOCKS: usize = 10000;

            while !self.check(&Token::RBrace) && !self.is_at_end() && bb_count < MAX_BASIC_BLOCKS {
                bb_count += 1;
                if let Some(bb) = self.parse_basic_block()? {
                    function.add_basic_block(bb);
                } else {
                    break;
                }
            }

            if bb_count >= MAX_BASIC_BLOCKS {
                return Err(ParseError::InvalidSyntax {
                    message: format!("Function exceeded maximum basic block count ({}), possible infinite loop", MAX_BASIC_BLOCKS),
                    position: self.current,
                });
            }

            self.consume(&Token::RBrace)?;
        }

        Ok(function)
    }

    fn parse_basic_block(&mut self) -> ParseResult<Option<BasicBlock>> {
        // Check for label - can be LocalIdent or keyword token followed by colon
        let name = if let Some(token) = self.peek().cloned() {
            if self.peek_ahead(1) == Some(&Token::Colon) {
                // Extract label name from various token types
                let label_name = match token {
                    Token::LocalIdent(n) => Some(n),
                    Token::Identifier(n) => Some(n), // Bare identifiers like BB1, then, etc.
                    Token::Integer(n) => Some(n.to_string()), // Numeric labels like 1:, 2:
                    // Common keywords that can be used as labels
                    Token::Entry => Some("entry".to_string()),
                    Token::Cleanup => Some("cleanup".to_string()),
                    Token::Catch => Some("catch".to_string()),
                    Token::Filter => Some("filter".to_string()),
                    Token::True => Some("true".to_string()),
                    Token::False => Some("false".to_string()),
                    // Instruction keywords that can be used as labels
                    Token::Call => Some("call".to_string()),
                    Token::Ret => Some("ret".to_string()),
                    Token::Br => Some("br".to_string()),
                    Token::Switch => Some("switch".to_string()),
                    Token::Add => Some("add".to_string()),
                    Token::Sub => Some("sub".to_string()),
                    Token::Mul => Some("mul".to_string()),
                    Token::Load => Some("load".to_string()),
                    Token::Store => Some("store".to_string()),
                    Token::Alloca => Some("alloca".to_string()),
                    // Any other token followed by colon is not a valid label
                    _ => None,
                };

                if let Some(name) = label_name {
                    self.advance(); // consume label token
                    self.advance(); // consume colon
                    Some(name)
                } else {
                    // Not a recognized label token but followed by colon
                    // This shouldn't happen, but skip both tokens to avoid infinite loop
                    self.advance(); // skip unknown token
                    self.advance(); // skip colon
                    Some(format!("unknown_label_{}", self.current))
                }
            } else {
                // Entry block without label
                None
            }
        } else {
            None
        };

        // If we didn't find a label and we're at end of function, return None
        if name.is_none() && (self.check(&Token::RBrace) || self.is_at_end()) {
            return Ok(None);
        }

        let bb = BasicBlock::new(name);

        // Parse instructions with iteration limit to prevent infinite loops
        let mut inst_count = 0;
        const MAX_INSTRUCTIONS_PER_BLOCK: usize = 10000;

        loop {
            if inst_count >= MAX_INSTRUCTIONS_PER_BLOCK {
                return Err(ParseError::InvalidSyntax {
                    message: format!("Basic block exceeded maximum instruction count ({}), possible infinite loop", MAX_INSTRUCTIONS_PER_BLOCK),
                    position: self.current,
                });
            }
            inst_count += 1;

            // Stop if we hit next label, closing brace, or EOF
            if self.check(&Token::RBrace) || self.is_at_end() {
                break;
            }

            // Check if next token is a label (any token followed by colon)
            // In LLVM IR, any identifier-like token can be a label, including keywords
            if self.peek_ahead(1) == Some(&Token::Colon) {
                break;
            }

            // Parse instruction
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
        // Skip calling convention modifiers (tail, musttail, notail)
        if let Some(Token::Identifier(id)) = self.peek() {
            if id == "tail" || id == "musttail" || id == "notail" {
                self.advance(); // skip the modifier
            }
        }

        // Check for result assignment: %name = ...
        let _result_name = if let Some(Token::LocalIdent(n)) = self.peek().cloned() {
            if self.peek_ahead(1) == Some(&Token::Equal) {
                self.advance(); // consume ident
                self.advance(); // consume =
                Some(n)
            } else {
                None
            }
        } else {
            None
        };

        // Parse instruction opcode
        let opcode = if let Some(op) = self.parse_opcode()? {
            op
        } else {
            return Ok(None);
        };

        // Parse operands (simplified for now)
        let operands = self.parse_instruction_operands(opcode)?;

        // Skip instruction-level attributes that come after operands (nounwind, readonly, etc.)
        self.skip_instruction_level_attributes();

        // Skip metadata attachments after instructions (,!dbg !0, !prof !1, etc.)
        while self.match_token(&Token::Comma) {
            if self.is_metadata_token() {
                // Skip metadata name like !dbg
                self.skip_metadata();
                // Skip metadata value like !0
                if self.is_metadata_token() {
                    self.skip_metadata();
                }
            } else {
                // Not metadata, put comma back and stop
                self.current -= 1;
                break;
            }
        }

        Ok(Some(Instruction::new(opcode, operands, None)))
    }

    fn parse_opcode(&mut self) -> ParseResult<Option<Opcode>> {
        let token = self.peek().ok_or(ParseError::UnexpectedEOF)?;

        let opcode = match token {
            Token::Ret => { self.advance(); Opcode::Ret }
            Token::Br => { self.advance(); Opcode::Br }
            Token::Switch => { self.advance(); Opcode::Switch }
            Token::IndirectBr => { self.advance(); Opcode::IndirectBr }
            Token::Invoke => { self.advance(); Opcode::Invoke }
            Token::Resume => { self.advance(); Opcode::Resume }
            Token::Unreachable => { self.advance(); Opcode::Unreachable }
            Token::CleanupRet => { self.advance(); Opcode::CleanupRet }
            Token::CatchRet => { self.advance(); Opcode::CatchRet }
            Token::CatchSwitch => { self.advance(); Opcode::CatchSwitch }
            Token::CallBr => { self.advance(); Opcode::CallBr }
            Token::FNeg => { self.advance(); Opcode::FNeg }
            Token::Add => { self.advance(); Opcode::Add }
            Token::FAdd => { self.advance(); Opcode::FAdd }
            Token::Sub => { self.advance(); Opcode::Sub }
            Token::FSub => { self.advance(); Opcode::FSub }
            Token::Mul => { self.advance(); Opcode::Mul }
            Token::FMul => { self.advance(); Opcode::FMul }
            Token::UDiv => { self.advance(); Opcode::UDiv }
            Token::SDiv => { self.advance(); Opcode::SDiv }
            Token::FDiv => { self.advance(); Opcode::FDiv }
            Token::URem => { self.advance(); Opcode::URem }
            Token::SRem => { self.advance(); Opcode::SRem }
            Token::FRem => { self.advance(); Opcode::FRem }
            Token::Shl => { self.advance(); Opcode::Shl }
            Token::LShr => { self.advance(); Opcode::LShr }
            Token::AShr => { self.advance(); Opcode::AShr }
            Token::And => { self.advance(); Opcode::And }
            Token::Or => { self.advance(); Opcode::Or }
            Token::Xor => { self.advance(); Opcode::Xor }
            Token::ExtractElement => { self.advance(); Opcode::ExtractElement }
            Token::InsertElement => { self.advance(); Opcode::InsertElement }
            Token::ShuffleVector => { self.advance(); Opcode::ShuffleVector }
            Token::ExtractValue => { self.advance(); Opcode::ExtractValue }
            Token::InsertValue => { self.advance(); Opcode::InsertValue }
            Token::Alloca => { self.advance(); Opcode::Alloca }
            Token::Load => { self.advance(); Opcode::Load }
            Token::Store => { self.advance(); Opcode::Store }
            Token::GetElementPtr => { self.advance(); Opcode::GetElementPtr }
            Token::Fence => { self.advance(); Opcode::Fence }
            Token::AtomicCmpXchg => { self.advance(); Opcode::AtomicCmpXchg }
            Token::AtomicRMW => { self.advance(); Opcode::AtomicRMW }
            Token::Trunc => { self.advance(); Opcode::Trunc }
            Token::ZExt => { self.advance(); Opcode::ZExt }
            Token::SExt => { self.advance(); Opcode::SExt }
            Token::FPToUI => { self.advance(); Opcode::FPToUI }
            Token::FPToSI => { self.advance(); Opcode::FPToSI }
            Token::UIToFP => { self.advance(); Opcode::UIToFP }
            Token::SIToFP => { self.advance(); Opcode::SIToFP }
            Token::FPTrunc => { self.advance(); Opcode::FPTrunc }
            Token::FPExt => { self.advance(); Opcode::FPExt }
            Token::PtrToInt => { self.advance(); Opcode::PtrToInt }
            Token::IntToPtr => { self.advance(); Opcode::IntToPtr }
            Token::BitCast => { self.advance(); Opcode::BitCast }
            Token::AddrSpaceCast => { self.advance(); Opcode::AddrSpaceCast }
            Token::ICmp => { self.advance(); Opcode::ICmp }
            Token::FCmp => { self.advance(); Opcode::FCmp }
            Token::Phi => { self.advance(); Opcode::PHI }
            Token::Call => { self.advance(); Opcode::Call }
            Token::Select => { self.advance(); Opcode::Select }
            Token::VAArg => { self.advance(); Opcode::VAArg }
            Token::LandingPad => { self.advance(); Opcode::LandingPad }
            _ => return Ok(None),
        };

        Ok(Some(opcode))
    }

    fn parse_instruction_operands(&mut self, opcode: Opcode) -> ParseResult<Vec<Value>> {
        let mut operands = Vec::new();

        // Parse based on instruction type
        match opcode {
            Opcode::Ret => {
                // ret void or ret type value
                if self.match_token(&Token::Void) {
                    // void return
                } else {
                    let _ty = self.parse_type()?;
                    let _val = self.parse_value()?;
                }
            }
            Opcode::Br => {
                // br label %dest or br i1 %cond, label %iftrue, label %iffalse
                if self.match_token(&Token::Label) {
                    let _dest = self.expect_local_ident()?;
                } else {
                    let _cond_ty = self.parse_type()?;
                    let _cond = self.parse_value()?;
                    self.consume(&Token::Comma)?;
                    self.consume(&Token::Label)?;
                    let _true_dest = self.expect_local_ident()?;
                    self.consume(&Token::Comma)?;
                    self.consume(&Token::Label)?;
                    let _false_dest = self.expect_local_ident()?;
                }
            }
            Opcode::CallBr => {
                // callbr type asm sideeffect "...", "..."() to label %normal [label %indirect1, ...]
                // Skip all tokens until we find a label (next basic block) or end of statement
                let mut skip_count = 0;
                const MAX_SKIP: usize = 200;

                while !self.is_at_end() && skip_count < MAX_SKIP {
                    // Check if we've reached a label (next basic block)
                    if self.peek_ahead(1) == Some(&Token::Colon) {
                        break;
                    }
                    // Check if we've reached next instruction
                    if self.check_local_ident() && self.peek_ahead(1) == Some(&Token::Equal) {
                        break;
                    }
                    // Check for common instruction opcodes
                    if self.peek().map(|t| matches!(t,
                        Token::Ret | Token::Br | Token::Switch | Token::Call |
                        Token::Store | Token::Load | Token::Alloca
                    )).unwrap_or(false) {
                        break;
                    }
                    // Check for end of function
                    if self.check(&Token::RBrace) {
                        break;
                    }

                    self.advance();
                    skip_count += 1;
                }
            }
            Opcode::Call => {
                // call [cc] [attrs] type [(param_types...)] @func(args...)
                // Skip calling convention first
                self.skip_linkage_and_visibility();
                // Skip return attributes (inreg, zeroext, etc.)
                self.skip_attributes();

                let _ret_ty = self.parse_type()?;

                // Check for optional function signature: (param_types...)
                if self.check(&Token::LParen) {
                    self.advance(); // consume '('
                    // Parse parameter types for function pointer
                    while !self.check(&Token::RParen) && !self.is_at_end() {
                        if self.match_token(&Token::Ellipsis) {
                            break; // varargs
                        }
                        self.parse_type()?;
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.consume(&Token::RParen)?;
                }

                let _func = self.parse_value()?;
                self.consume(&Token::LParen)?;
                let _args = self.parse_call_arguments()?;
                self.consume(&Token::RParen)?;

                // Handle operand bundles: ["bundle"(args...)]
                if self.check(&Token::LBracket) {
                    self.advance(); // consume [
                    while !self.check(&Token::RBracket) && !self.is_at_end() {
                        // Skip bundle name (string literal)
                        if let Some(Token::StringLit(_)) = self.peek() {
                            self.advance();
                        }
                        // Skip bundle arguments: (args...)
                        if self.check(&Token::LParen) {
                            self.advance();
                            // Skip all arguments
                            while !self.check(&Token::RParen) && !self.is_at_end() {
                                self.advance();
                            }
                            self.match_token(&Token::RParen);
                        }
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.match_token(&Token::RBracket);
                }
            }
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::UDiv | Opcode::SDiv |
            Opcode::URem | Opcode::SRem | Opcode::Shl | Opcode::LShr | Opcode::AShr |
            Opcode::And | Opcode::Or | Opcode::Xor |
            Opcode::FAdd | Opcode::FSub | Opcode::FMul | Opcode::FDiv | Opcode::FRem => {
                // Binary ops: op [flags] type op1, op2
                self.skip_instruction_flags();
                let _ty = self.parse_type()?;
                let _op1 = self.parse_value()?;
                self.consume(&Token::Comma)?;
                let _op2 = self.parse_value()?;
            }
            Opcode::Alloca => {
                // alloca [inalloca] type [, type NumElements] [, align N] [, addrspace(N)]
                // Skip inalloca keyword (it's Token::Inalloca, not an identifier)
                self.match_token(&Token::Inalloca);

                let _ty = self.parse_type()?;

                // Handle optional attributes in any order
                while self.match_token(&Token::Comma) {
                    if self.match_token(&Token::Align) {
                        if let Some(Token::Integer(_)) = self.peek() {
                            self.advance();
                        }
                    } else if self.match_token(&Token::Addrspace) {
                        self.consume(&Token::LParen)?;
                        if let Some(Token::Integer(_)) = self.peek() {
                            self.advance();
                        }
                        self.consume(&Token::RParen)?;
                    } else if self.is_metadata_token() {
                        // Metadata attachment - skip all metadata and stop parsing
                        // Metadata can be space-separated: !foo !0 or comma-separated: !foo, !bar
                        self.skip_metadata();
                        while self.is_metadata_token() ||
                              (self.match_token(&Token::Comma) && self.is_metadata_token()) {
                            self.skip_metadata();
                        }
                        break;
                    } else if !self.check_type_token() {
                        // Skip unknown attributes
                        self.advance();
                    } else {
                        // This is array size: type value
                        let _size_ty = self.parse_type()?;
                        let _size_val = self.parse_value()?;
                    }
                }

                // Check for metadata even without preceding comma
                if self.is_metadata_token() {
                    self.skip_metadata();
                    while self.is_metadata_token() ||
                          (self.match_token(&Token::Comma) && self.is_metadata_token()) {
                        self.skip_metadata();
                    }
                }
            }
            Opcode::Load => {
                // load [atomic] [volatile] type, ptr %ptr [unordered|monotonic|acquire|...] [, align ...]
                // Skip atomic and volatile modifiers
                self.match_token(&Token::Atomic);
                self.match_token(&Token::Volatile);

                let _ty = self.parse_type()?;
                self.consume(&Token::Comma)?;
                let _ptr_ty = self.parse_type()?;
                let _ptr = self.parse_value()?;

                // Skip memory ordering and other attributes
                self.skip_load_store_attributes();
            }
            Opcode::Store => {
                // store [atomic] [volatile] type %val, ptr %ptr [, align ...]
                // Skip atomic and volatile modifiers
                self.match_token(&Token::Atomic);
                self.match_token(&Token::Volatile);

                let _val_ty = self.parse_type()?;
                let _val = self.parse_value()?;
                self.consume(&Token::Comma)?;
                let _ptr_ty = self.parse_type()?;
                let _ptr = self.parse_value()?;
                // Skip attributes
                self.skip_load_store_attributes();
            }
            Opcode::GetElementPtr => {
                // getelementptr [inbounds] type, ptr %ptr, indices...
                self.match_token(&Token::Inbounds);
                let _ty = self.parse_type()?;
                self.consume(&Token::Comma)?;
                let _ptr_ty = self.parse_type()?;
                let _ptr = self.parse_value()?;
                // Parse indices
                while self.match_token(&Token::Comma) {
                    let _idx_ty = self.parse_type()?;
                    let _idx = self.parse_value()?;
                }
            }
            Opcode::ICmp | Opcode::FCmp => {
                // icmp/fcmp predicate type op1, op2
                self.parse_comparison_predicate()?;
                let _ty = self.parse_type()?;
                let _op1 = self.parse_value()?;
                self.consume(&Token::Comma)?;
                let _op2 = self.parse_value()?;
            }
            Opcode::PHI => {
                // phi type [ val1, %bb1 ], [ val2, %bb2 ], ...
                let _ty = self.parse_type()?;
                while !self.is_at_end() {
                    if !self.match_token(&Token::LBracket) {
                        break;
                    }
                    let _val = self.parse_value()?;
                    self.consume(&Token::Comma)?;
                    let _bb = self.expect_local_ident()?;
                    self.consume(&Token::RBracket)?;
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            Opcode::Trunc | Opcode::ZExt | Opcode::SExt | Opcode::FPTrunc | Opcode::FPExt |
            Opcode::FPToUI | Opcode::FPToSI | Opcode::UIToFP | Opcode::SIToFP |
            Opcode::PtrToInt | Opcode::IntToPtr | Opcode::BitCast | Opcode::AddrSpaceCast => {
                // cast type1 %val to type2
                let _src_ty = self.parse_type()?;
                let _val = self.parse_value()?;
                self.consume(&Token::To)?;
                let _dest_ty = self.parse_type()?;
            }
            Opcode::Select => {
                // select i1 %cond, type %val1, type %val2
                let _cond_ty = self.parse_type()?;
                let _cond = self.parse_value()?;
                self.consume(&Token::Comma)?;
                let _ty1 = self.parse_type()?;
                let _val1 = self.parse_value()?;
                self.consume(&Token::Comma)?;
                let _ty2 = self.parse_type()?;
                let _val2 = self.parse_value()?;
            }
            Opcode::AtomicCmpXchg => {
                // cmpxchg [weak] [volatile] ptr <pointer>, type <cmp>, type <new> [syncscope] <ordering> <ordering>
                self.match_token(&Token::Weak);
                self.match_token(&Token::Volatile);

                // Parse pointer type and value
                let _ptr_ty = self.parse_type()?;
                let _ptr = self.parse_value()?;
                self.consume(&Token::Comma)?;

                // Parse compare type and value
                let _cmp_ty = self.parse_type()?;
                let _cmp = self.parse_value()?;
                self.consume(&Token::Comma)?;

                // Parse new type and value
                let _new_ty = self.parse_type()?;
                let _new = self.parse_value()?;

                // Skip syncscope if present
                self.skip_syncscope();

                // Parse two memory orderings (as keyword tokens, not identifiers)
                self.skip_memory_ordering();
                self.skip_memory_ordering();
            }
            Opcode::AtomicRMW => {
                // atomicrmw [volatile] <operation> ptr <pointer>, type <value> [syncscope] <ordering>
                self.match_token(&Token::Volatile);

                // Parse operation (add, sub, xchg, etc.)
                // These can be opcodes or identifiers
                self.advance(); // Skip the operation token (whatever it is)

                // Parse pointer type and value
                let _ptr_ty = self.parse_type()?;
                let _ptr = self.parse_value()?;
                self.consume(&Token::Comma)?;

                // Parse value type and value
                let _val_ty = self.parse_type()?;
                let _val = self.parse_value()?;

                // Skip syncscope if present
                self.skip_syncscope();

                // Parse ordering (as keyword token, not identifier)
                self.skip_memory_ordering();
            }
            _ => {
                // For other instructions, skip to end of line or next instruction
                // Skip until we find something that looks like the next instruction/statement
                let mut skip_count = 0;
                const MAX_SKIP_TOKENS: usize = 100;

                while !self.is_at_end() && skip_count < MAX_SKIP_TOKENS {
                    // Stop if we hit what looks like a new statement
                    if self.check_local_ident() && self.peek_ahead(1) == Some(&Token::Equal) {
                        // Next instruction assignment
                        break;
                    }
                    // Check for any token followed by colon (labels)
                    if self.peek_ahead(1) == Some(&Token::Colon) {
                        // Label (can be LocalIdent, Identifier, Integer, or keyword)
                        break;
                    }
                    if self.peek().map(|t| matches!(t,
                        Token::Ret | Token::Br | Token::Switch | Token::Call |
                        Token::Store | Token::Load | Token::Add | Token::Sub |
                        Token::Mul | Token::Alloca | Token::GetElementPtr |
                        Token::ICmp | Token::FCmp | Token::Phi
                    )).unwrap_or(false) {
                        // Next instruction opcode
                        break;
                    }
                    if self.check(&Token::RBrace) {
                        // End of function
                        break;
                    }

                    self.advance();
                    skip_count += 1;
                }
            }
        }

        Ok(operands)
    }

    fn parse_comparison_predicate(&mut self) -> ParseResult<()> {
        // Parse comparison predicate (eq, ne, ugt, etc.)
        if self.match_token(&Token::Eq) || self.match_token(&Token::Ne) ||
           self.match_token(&Token::Ugt) || self.match_token(&Token::Uge) ||
           self.match_token(&Token::Ult) || self.match_token(&Token::Ule) ||
           self.match_token(&Token::Sgt) || self.match_token(&Token::Sge) ||
           self.match_token(&Token::Slt) || self.match_token(&Token::Sle) ||
           self.match_token(&Token::Oeq) || self.match_token(&Token::Ogt) ||
           self.match_token(&Token::Oge) || self.match_token(&Token::Olt) ||
           self.match_token(&Token::Ole) || self.match_token(&Token::One) ||
           self.match_token(&Token::Ord) || self.match_token(&Token::Uno) ||
           self.match_token(&Token::Ueq) {
            Ok(())
        } else {
            Err(ParseError::InvalidSyntax {
                message: "Expected comparison predicate".to_string(),
                position: self.current,
            })
        }
    }

    fn skip_metadata(&mut self) {
        // Skip metadata reference: !{...}, !0, !DIExpression(), !foo, etc.
        // The lexer combines !foo into Token::MetadataIdent("foo"), so we need to handle both cases

        // Case 1: Token::MetadataIdent - lexer already combined ! with identifier/number
        if let Some(Token::MetadataIdent(_)) = self.peek() {
            self.advance();
            // Check if followed by parentheses like !DIExpression(...)
            if self.check(&Token::LParen) {
                self.advance();
                let mut depth = 1;
                while depth > 0 && !self.is_at_end() {
                    if self.check(&Token::LParen) {
                        depth += 1;
                    } else if self.check(&Token::RParen) {
                        depth -= 1;
                    }
                    self.advance();
                }
            }
        }
        // Case 2: Token::Exclaim followed by something else (like !{...})
        else if self.match_token(&Token::Exclaim) {
            if self.check(&Token::LBrace) {
                // !{...}
                self.advance();
                let mut depth = 1;
                while depth > 0 && !self.is_at_end() {
                    if self.check(&Token::LBrace) {
                        depth += 1;
                    } else if self.check(&Token::RBrace) {
                        depth -= 1;
                    }
                    self.advance();
                }
            } else if let Some(Token::Identifier(_)) = self.peek() {
                // !DIExpression() - when lexer didn't combine them
                self.advance();
                if self.check(&Token::LParen) {
                    self.advance();
                    let mut depth = 1;
                    while depth > 0 && !self.is_at_end() {
                        if self.check(&Token::LParen) {
                            depth += 1;
                        } else if self.check(&Token::RParen) {
                            depth -= 1;
                        }
                        self.advance();
                    }
                }
            } else if let Some(Token::Integer(_)) = self.peek() {
                // !0, !1, etc. - when lexer didn't combine them
                self.advance();
            } else if let Some(Token::StringLit(_)) = self.peek() {
                // !"string" - metadata string literal
                self.advance();
            }
        }
    }

    fn parse_call_arguments(&mut self) -> ParseResult<Vec<(Type, Value)>> {
        let mut args = Vec::new();

        while !self.check(&Token::RParen) && !self.is_at_end() {
            // Handle metadata arguments specially: metadata i32 0 or metadata !{}
            if self.match_token(&Token::Metadata) {
                // Check for various metadata forms
                if let Some(Token::MetadataIdent(_)) = self.peek() {
                    // metadata !DIExpression() or !0
                    self.advance(); // consume the metadata ident
                    // If followed by parens, skip them
                    if self.check(&Token::LParen) {
                        self.advance();
                        let mut depth = 1;
                        while depth > 0 && !self.is_at_end() {
                            if self.check(&Token::LParen) {
                                depth += 1;
                            } else if self.check(&Token::RParen) {
                                depth -= 1;
                            }
                            self.advance();
                        }
                    }
                } else if self.check(&Token::Exclaim) {
                    // metadata !{} - literal metadata
                    self.skip_metadata();
                } else {
                    // metadata i32 0 - parse type and value
                    let _ty = self.parse_type()?;
                    let _val = self.parse_value()?;
                }
                if !self.match_token(&Token::Comma) {
                    break;
                }
                continue;
            }

            let ty = self.parse_type()?;

            // Skip parameter attributes (byval, sret, noundef, allocalign, etc.)
            loop {
                // Attributes without type parameters
                if self.match_token(&Token::Inreg) ||
                   self.match_token(&Token::Noalias) ||
                   self.match_token(&Token::Nocapture) ||
                   self.match_token(&Token::Nest) ||
                   self.match_token(&Token::Zeroext) ||
                   self.match_token(&Token::Signext) ||
                   self.match_token(&Token::Immarg) {
                    continue;
                }

                // Attributes with optional type parameters: byval(type), sret(type), inalloca(type)
                if self.match_token(&Token::Byval) ||
                   self.match_token(&Token::Sret) ||
                   self.match_token(&Token::Inalloca) {
                    // Handle optional (type) syntax
                    if self.check(&Token::LParen) {
                        self.advance();
                        // Skip tokens until )
                        while !self.check(&Token::RParen) && !self.is_at_end() {
                            self.advance();
                        }
                        self.match_token(&Token::RParen);
                    }
                    continue;
                }

                // Handle identifier-based attributes with type parameters: byref(type), elementtype(type)
                if let Some(Token::Identifier(attr)) = self.peek() {
                    if matches!(attr.as_str(), "byref" | "elementtype") {
                        self.advance();
                        if self.check(&Token::LParen) {
                            self.advance();
                            while !self.check(&Token::RParen) && !self.is_at_end() {
                                self.advance();
                            }
                            self.match_token(&Token::RParen);
                        }
                        continue;
                    }
                }

                // Handle align N
                if self.match_token(&Token::Align) {
                    if let Some(Token::Integer(_)) = self.peek() {
                        self.advance();
                    }
                    continue;
                }

                // Handle identifier-based attributes
                if let Some(Token::Identifier(attr)) = self.peek() {
                    if matches!(attr.as_str(), "noundef" | "nonnull" | "readonly" | "writeonly" |
                                              "allocalign" | "allocsize" | "returned") {
                        self.advance();
                        continue;
                    }
                }

                // No more attributes
                break;
            }

            let val = self.parse_value()?;
            args.push((ty, val));

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(args)
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        let token = self.peek().cloned().ok_or(ParseError::UnexpectedEOF)?;

        let base_type = match token {
            Token::Void => {
                self.advance();
                Ok(self.context.void_type())
            }
            Token::IntType(bits) => {
                self.advance();
                Ok(self.context.int_type(bits))
            }
            Token::Half | Token::Bfloat | Token::Float | Token::Double |
            Token::X86_fp80 | Token::Fp128 | Token::Ppc_fp128 => {
                let is_half = matches!(token, Token::Half);
                let is_double = matches!(token, Token::Double);
                self.advance();
                if is_half {
                    Ok(self.context.half_type())
                } else if is_double {
                    Ok(self.context.double_type())
                } else {
                    Ok(self.context.float_type())
                }
            }
            Token::Ptr => {
                self.advance();
                // Check if this is a function pointer type: ptr (...)
                if self.check(&Token::LParen) {
                    self.advance(); // consume '('

                    // Parse function parameter types
                    let mut param_types = Vec::new();

                    while !self.check(&Token::RParen) && !self.is_at_end() {
                        param_types.push(self.parse_type()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }

                    self.consume(&Token::RParen)?;

                    // Function pointers have void return type by default
                    let return_type = self.context.void_type();
                    let func_type = self.context.function_type(return_type, param_types, false);
                    Ok(self.context.ptr_type(func_type))
                } else {
                    // Check if followed by addrspace modifier
                    if self.check(&Token::Addrspace) {
                        self.advance(); // consume 'addrspace'
                        self.consume(&Token::LParen)?;
                        // Parse address space number
                        if let Some(Token::Integer(_n)) = self.peek() {
                            self.advance();
                        }
                        self.consume(&Token::RParen)?;
                    }
                    // Modern LLVM uses opaque pointers (ptr)
                    Ok(self.context.ptr_type(self.context.int8_type()))
                }
            }
            Token::Label => {
                self.advance();
                Ok(self.context.label_type())
            }
            Token::Token => {
                self.advance();
                // Token type for statepoints/gc - use metadata type as placeholder
                Ok(self.context.metadata_type())
            }
            Token::Metadata => {
                self.advance();
                Ok(self.context.metadata_type())
            }
            Token::Target => {
                // target("typename") - target-specific types
                self.advance(); // consume 'target'
                self.consume(&Token::LParen)?;
                if let Some(Token::StringLit(_)) = self.peek() {
                    self.advance(); // consume type name string
                }
                self.consume(&Token::RParen)?;
                // Return opaque type placeholder
                Ok(self.context.int8_type())
            }
            Token::LBracket => {
                // Array type: [ size x type ]
                self.advance();
                let size = if let Some(Token::Integer(n)) = self.peek() {
                    let size = *n as u64;
                    self.advance();
                    size
                } else {
                    return Err(ParseError::InvalidSyntax {
                        message: "Expected array size".to_string(),
                        position: self.current,
                    });
                };
                self.consume(&Token::X)?; // 'x'
                let elem_ty = self.parse_type()?;
                self.consume(&Token::RBracket)?;
                Ok(self.context.array_type(elem_ty, size as usize))
            }
            Token::LBrace => {
                // Struct type: { type1, type2, ... }
                self.advance();
                let mut field_types = Vec::new();
                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    let ty = self.parse_type()?;
                    field_types.push(ty);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.consume(&Token::RBrace)?;
                Ok(crate::types::Type::struct_type(&self.context, field_types, None))
            }
            Token::LAngle => {
                // Vector type: < size x type > or scalable: < vscale x size x type >
                self.advance();

                // Check for vscale (scalable vector)
                let _is_scalable = if let Some(Token::Identifier(id)) = self.peek() {
                    if id == "vscale" {
                        self.advance();
                        self.consume(&Token::X)?; // consume 'x' after vscale
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                let size = if let Some(Token::Integer(n)) = self.peek() {
                    let size = *n as u64;
                    self.advance();
                    size
                } else {
                    return Err(ParseError::InvalidSyntax {
                        message: "Expected vector size".to_string(),
                        position: self.current,
                    });
                };
                self.consume(&Token::X)?; // 'x'
                let elem_ty = self.parse_type()?;
                self.consume(&Token::RAngle)?;
                // For now, treat scalable vectors like regular vectors
                Ok(self.context.vector_type(elem_ty, size as usize))
            }
            Token::LocalIdent(_) => {
                // Type reference like %TypeName
                self.advance();
                // For now, treat as opaque type
                Ok(self.context.void_type())
            }
            _ => {
                Err(ParseError::UnknownType {
                    type_name: format!("{:?}", token),
                    position: self.current,
                })
            }
        }?;

        // Check for function type syntax: type(params...)
        if self.check(&Token::LParen) {
            self.advance(); // consume '('

            let mut param_types = Vec::new();
            while !self.check(&Token::RParen) && !self.is_at_end() {
                // Check for varargs
                if self.check(&Token::Ellipsis) {
                    self.advance();
                    break;
                }
                let param_ty = self.parse_type()?;
                param_types.push(param_ty);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            self.consume(&Token::RParen)?;

            // base_type is the return type, create function type
            let func_type = self.context.function_type(base_type, param_types, false);
            return Ok(func_type);
        }

        // Check for old-style typed pointer syntax: type addrspace(n)* or type*
        // Skip optional addrspace modifier
        if self.check(&Token::Addrspace) {
            self.advance(); // consume 'addrspace'
            self.consume(&Token::LParen)?;
            if let Some(Token::Integer(_)) = self.peek() {
                self.advance();
            }
            self.consume(&Token::RParen)?;
        }

        // Check for * to make it a pointer
        if self.check(&Token::Star) {
            self.advance(); // consume '*'
            return Ok(self.context.ptr_type(base_type));
        }

        Ok(base_type)
    }

    fn parse_value(&mut self) -> ParseResult<Value> {
        let token = self.peek().ok_or(ParseError::UnexpectedEOF)?;

        match token {
            Token::Identifier(id) if id == "asm" => {
                // Inline assembly: asm [volatile] [sideeffect] "asm code", "constraints"
                // Note: The () after constraints is part of the call instruction, not the asm value
                self.advance(); // consume 'asm'

                // Skip optional keywords
                while let Some(Token::Identifier(kw)) = self.peek() {
                    if matches!(kw.as_str(), "volatile" | "sideeffect" | "alignstack" | "inteldialect") {
                        self.advance();
                    } else {
                        break;
                    }
                }

                // Skip asm string
                if let Some(Token::StringLit(_)) = self.peek() {
                    self.advance();
                }

                // Skip comma and constraints string
                if self.match_token(&Token::Comma) {
                    if let Some(Token::StringLit(_)) = self.peek() {
                        self.advance();
                    }
                }

                // Return placeholder value (don't consume the call arguments here)
                Ok(Value::undef(self.context.void_type()))
            }
            Token::Identifier(id) if id == "splat" => {
                // Vector splat: splat (type value)
                self.advance(); // consume 'splat'
                self.consume(&Token::LParen)?;
                let _ty = self.parse_type()?;
                let _val = self.parse_value()?;
                self.consume(&Token::RParen)?;
                // Return placeholder splat value
                Ok(Value::zero_initializer(self.context.void_type()))
            }
            Token::LocalIdent(name) => {
                let name = name.clone();
                self.advance();
                // Create a placeholder instruction value for local variables
                Ok(Value::instruction(self.context.void_type(), Opcode::Add, Some(name)))
            }
            Token::GlobalIdent(name) => {
                let name = name.clone();
                self.advance();
                // Create a global variable value
                Ok(Value::new(self.context.void_type(), crate::value::ValueKind::GlobalVariable { is_constant: false }, Some(name)))
            }
            Token::Integer(n) => {
                let n = *n;
                self.advance();
                Ok(Value::const_int(self.context.int32_type(), n as i64, None))
            }
            Token::Float64(f) => {
                let f = *f;
                self.advance();
                Ok(Value::const_float(self.context.double_type(), f, None))
            }
            Token::True => {
                self.advance();
                Ok(Value::const_int(self.context.bool_type(), 1, None))
            }
            Token::False => {
                self.advance();
                Ok(Value::const_int(self.context.bool_type(), 0, None))
            }
            Token::Null => {
                self.advance();
                Ok(Value::const_null(self.context.ptr_type(self.context.int8_type())))
            }
            Token::None => {
                self.advance();
                // 'none' is used with token type in GC intrinsics
                Ok(Value::undef(self.context.metadata_type()))
            }
            Token::Undef => {
                self.advance();
                Ok(Value::undef(self.context.void_type()))
            }
            Token::Poison => {
                self.advance();
                Ok(Value::undef(self.context.void_type())) // Treat poison like undef for now
            }
            Token::Zeroinitializer => {
                self.advance();
                Ok(Value::zero_initializer(self.context.void_type()))
            }
            Token::LAngle => {
                // Vector constant: < type val1, type val2, ... >
                self.advance(); // consume '<'
                // Parse vector elements
                while !self.check(&Token::RAngle) && !self.is_at_end() {
                    // Parse element type and value
                    let _elem_ty = self.parse_type()?;
                    let _elem_val = self.parse_value()?;
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.consume(&Token::RAngle)?;
                // Return placeholder vector constant
                Ok(Value::zero_initializer(self.context.void_type()))
            }
            Token::LBracket => {
                // Array constant: [type val1, type val2, ...]
                self.advance(); // consume '['
                while !self.check(&Token::RBracket) && !self.is_at_end() {
                    let _ty = self.parse_type()?;
                    let _val = self.parse_value()?;
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.consume(&Token::RBracket)?;
                // Return placeholder array constant
                Ok(Value::zero_initializer(self.context.void_type()))
            }
            Token::LBrace => {
                // Struct constant: { type val1, type val2, ... }
                self.advance(); // consume '{'
                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    let _ty = self.parse_type()?;
                    let _val = self.parse_value()?;
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.consume(&Token::RBrace)?;
                // Return placeholder struct constant
                Ok(Value::zero_initializer(self.context.void_type()))
            }
            // Constant expressions - instructions that can appear in constant contexts
            Token::PtrToInt | Token::IntToPtr | Token::BitCast | Token::AddrSpaceCast |
            Token::Trunc | Token::ZExt | Token::SExt | Token::FPTrunc | Token::FPExt |
            Token::FPToUI | Token::FPToSI | Token::UIToFP | Token::SIToFP |
            Token::GetElementPtr | Token::Sub | Token::Add | Token::Mul |
            Token::UDiv | Token::SDiv | Token::URem | Token::SRem |
            Token::Shl | Token::LShr | Token::AShr | Token::And | Token::Or | Token::Xor |
            Token::ICmp | Token::FCmp | Token::Select => {
                // Parse as constant expression
                self.parse_constant_expression()
            }
            _ => {
                Err(ParseError::InvalidSyntax {
                    message: format!("Expected value, found {:?}", token),
                    position: self.current,
                })
            }
        }
    }

    fn parse_constant_expression(&mut self) -> ParseResult<Value> {
        // Parse constant expressions like: ptrtoint (ptr @global to i32)
        let token = self.peek().ok_or(ParseError::UnexpectedEOF)?;

        // Determine the opcode
        let opcode = match token {
            Token::PtrToInt => Opcode::PtrToInt,
            Token::IntToPtr => Opcode::IntToPtr,
            Token::BitCast => Opcode::BitCast,
            Token::AddrSpaceCast => Opcode::AddrSpaceCast,
            Token::Trunc => Opcode::Trunc,
            Token::ZExt => Opcode::ZExt,
            Token::SExt => Opcode::SExt,
            Token::FPTrunc => Opcode::FPTrunc,
            Token::FPExt => Opcode::FPExt,
            Token::FPToUI => Opcode::FPToUI,
            Token::FPToSI => Opcode::FPToSI,
            Token::UIToFP => Opcode::UIToFP,
            Token::SIToFP => Opcode::SIToFP,
            Token::GetElementPtr => Opcode::GetElementPtr,
            Token::Sub => Opcode::Sub,
            Token::Add => Opcode::Add,
            Token::Mul => Opcode::Mul,
            Token::UDiv => Opcode::UDiv,
            Token::SDiv => Opcode::SDiv,
            Token::URem => Opcode::URem,
            Token::SRem => Opcode::SRem,
            Token::Shl => Opcode::Shl,
            Token::LShr => Opcode::LShr,
            Token::AShr => Opcode::AShr,
            Token::And => Opcode::And,
            Token::Or => Opcode::Or,
            Token::Xor => Opcode::Xor,
            Token::ICmp => Opcode::ICmp,
            Token::FCmp => Opcode::FCmp,
            Token::Select => Opcode::Select,
            _ => {
                return Err(ParseError::InvalidSyntax {
                    message: format!("Unexpected token in constant expression: {:?}", token),
                    position: self.current,
                });
            }
        };

        self.advance(); // consume opcode token

        // Parse the operands inside parentheses
        self.consume(&Token::LParen)?;

        // For cast operations: castop (srctype value to desttype)
        // For binary ops: binop (type val1, type val2)
        // For GEP: getelementptr (basetype, ptrtype ptrvalue, indices...)
        // For select: select (type cond, type val1, type val2)

        if matches!(opcode, Opcode::GetElementPtr) {
            // GEP is special: getelementptr (basetype, ptrtype ptrvalue, indextype indexvalue, ...)
            let _base_ty = self.parse_type()?;
            self.consume(&Token::Comma)?;
            let _ptr_ty = self.parse_type()?;
            let _ptr_val = self.parse_value()?;
            // Parse remaining indices
            while self.match_token(&Token::Comma) {
                let _idx_ty = self.parse_type()?;
                let _idx_val = self.parse_value()?;
            }
        } else {
            // Simplified parsing - just parse type and value, skip to closing paren
            // This allows the constant expression to be recognized without full semantic support
            let _src_ty = self.parse_type()?;
            let _src_val = self.parse_value()?;

            // Handle 'to' keyword for casts
            if matches!(opcode, Opcode::PtrToInt | Opcode::IntToPtr | Opcode::BitCast |
                               Opcode::Trunc | Opcode::ZExt | Opcode::SExt |
                               Opcode::FPTrunc | Opcode::FPExt | Opcode::FPToUI |
                               Opcode::FPToSI | Opcode::UIToFP | Opcode::SIToFP |
                               Opcode::AddrSpaceCast) {
                if self.match_token(&Token::To) {
                    let _dest_ty = self.parse_type()?;
                }
            } else if matches!(opcode, Opcode::Select) {
                // Select: select (type cond, type val1, type val2)
                if self.match_token(&Token::Comma) {
                    let _ty2 = self.parse_type()?;
                    let _val2 = self.parse_value()?;
                    if self.match_token(&Token::Comma) {
                        let _ty3 = self.parse_type()?;
                        let _val3 = self.parse_value()?;
                    }
                }
            } else if matches!(opcode, Opcode::ICmp | Opcode::FCmp) {
                // Comparison: icmp/fcmp (predicate type val1, type val2)
                // The first "type" we parsed might actually be a predicate, skip
                if self.match_token(&Token::Comma) {
                    let _ty2 = self.parse_type()?;
                    let _val2 = self.parse_value()?;
                }
            } else {
                // Binary operations and others - parse second operand if comma present
                if self.match_token(&Token::Comma) {
                    let _ty2 = self.parse_type()?;
                    let _val2 = self.parse_value()?;
                }
            }
        }

        self.consume(&Token::RParen)?;

        // Return a placeholder constant expression value
        Ok(Value::instruction(self.context.void_type(), opcode, Some("constexpr".to_string())))
    }

    fn parse_parameters(&mut self) -> ParseResult<Vec<(Type, String)>> {
        let mut params = Vec::new();

        while !self.check(&Token::RParen) && !self.is_at_end() {
            // Check for varargs (just ellipsis with no type)
            if self.check(&Token::Ellipsis) {
                self.advance();
                break;
            }

            let ty = self.parse_type()?;

            // Parse parameter attributes (readonly, etc.)
            self.skip_parameter_attributes();

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

    fn parse_parameter_types(&mut self) -> ParseResult<Vec<Type>> {
        let mut types = Vec::new();

        while !self.check(&Token::RParen) && !self.is_at_end() {
            // Check for varargs
            if self.check(&Token::Ellipsis) {
                self.advance();
                break;
            }

            let ty = self.parse_type()?;
            types.push(ty);

            // Skip parameter attributes
            self.skip_parameter_attributes();

            // Skip local ident if present
            if self.check_local_ident() {
                self.advance();
            }

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(types)
    }

    // Helper methods

    fn skip_linkage_and_visibility(&mut self) {
        loop {
            if self.match_token(&Token::Private) ||
               self.match_token(&Token::Internal) ||
               self.match_token(&Token::External) ||
               self.match_token(&Token::Weak) ||
               self.match_token(&Token::Linkonce) ||
               self.match_token(&Token::Linkonce_odr) ||
               self.match_token(&Token::Weak_odr) ||
               self.match_token(&Token::Available_externally) ||
               self.match_token(&Token::Extern_weak) ||
               self.match_token(&Token::Common) ||
               self.match_token(&Token::Appending) ||
               self.match_token(&Token::Hidden) ||
               self.match_token(&Token::Protected) ||
               self.match_token(&Token::Default) ||
               self.match_token(&Token::Dllimport) ||
               self.match_token(&Token::Dllexport) ||
               self.match_token(&Token::Unnamed_addr) ||
               self.match_token(&Token::Dso_local) ||
               self.match_token(&Token::Dso_preemptable) ||
               self.match_token(&Token::Thread_local) ||
               self.match_token(&Token::Local_unnamed_addr) ||
               // GPU calling conventions
               self.match_token(&Token::Amdgpu_kernel) ||
               self.match_token(&Token::Amdgpu_cs_chain) ||
               self.match_token(&Token::Amdgpu_ps) {
                // Keep consuming
                continue;
            }

            // Check for identifier-based calling conventions (e.g., amdgpu_cs_chain_preserve)
            if let Some(Token::Identifier(id)) = self.peek() {
                if id.starts_with("amdgpu_") || id.starts_with("spir_") ||
                   id.starts_with("aarch64_") || id == "cc" || id.starts_with("cc") {
                    self.advance();
                    continue;
                }
            }

            break;
        }

        // Handle addrspace modifier: addrspace(N)
        if self.match_token(&Token::Addrspace) {
            if self.match_token(&Token::LParen) {
                // Parse address space number
                if let Some(Token::Integer(_)) = self.peek() {
                    self.advance();
                }
                self.match_token(&Token::RParen);
            }
        }
    }

    fn skip_attributes(&mut self) {
        // Skip numbered attribute groups (#0, #1, etc.) and attribute keywords
        loop {
            if self.check(&Token::Hash) || self.check_attr_group_id() {
                self.advance();
                continue;
            }

            if self.match_token(&Token::Inreg) ||
               self.match_token(&Token::Zeroext) ||
               self.match_token(&Token::Signext) ||
               self.match_token(&Token::Noalias) ||
               self.match_token(&Token::Nocapture) ||
               self.match_token(&Token::Nest) {
                continue;
            }

            // Handle attributes with optional type parameters: byval(type), sret(type), byref(type), inalloca(type)
            if self.match_token(&Token::Byval) ||
               self.match_token(&Token::Sret) ||
               self.match_token(&Token::Inalloca) {
                if self.check(&Token::LParen) {
                    self.advance();
                    // Skip the type - just consume tokens until )
                    while !self.check(&Token::RParen) && !self.is_at_end() {
                        self.advance();
                    }
                    self.match_token(&Token::RParen);
                }
                continue;
            }

            // Handle byref attribute (identifier-based with type parameter)
            if let Some(Token::Identifier(attr)) = self.peek() {
                if attr == "byref" {
                    self.advance();
                    if self.check(&Token::LParen) {
                        self.advance();
                        // Skip the type - just consume tokens until )
                        while !self.check(&Token::RParen) && !self.is_at_end() {
                            self.advance();
                        }
                        self.match_token(&Token::RParen);
                    }
                    continue;
                }
            }

            // Handle identifier-based attributes (noundef, nonnull, etc.)
            if let Some(Token::Identifier(attr)) = self.peek() {
                if matches!(attr.as_str(), "noundef" | "nonnull" | "readonly" | "writeonly" |
                                          "readnone" | "returned" | "noreturn" | "nounwind" |
                                          "allocalign" | "allocsize" | "initializes") {
                    self.advance();
                    // Some attributes have parameters in parentheses (possibly nested like initializes((0, 4)))
                    if self.check(&Token::LParen) {
                        self.advance(); // consume first (
                        let mut depth = 1;
                        while depth > 0 && !self.is_at_end() {
                            if self.check(&Token::LParen) {
                                depth += 1;
                                self.advance();
                            } else if self.check(&Token::RParen) {
                                depth -= 1;
                                self.advance();
                            } else {
                                self.advance();
                            }
                        }
                    }
                    continue;
                }
            }

            // Handle align followed by integer: align 16
            if self.match_token(&Token::Align) {
                if let Some(Token::Integer(_)) = self.peek() {
                    self.advance();
                }
                continue;
            }

            // No more attributes
            break;
        }
    }

    fn skip_parameter_attributes(&mut self) {
        // Skip attributes like readonly, nonnull, byval(type), etc.
        let mut skip_count = 0;
        const MAX_ATTR_SKIP: usize = 50;

        while !self.is_at_end() && skip_count < MAX_ATTR_SKIP {
            if self.check_local_ident() || self.check(&Token::Comma) ||
               self.check(&Token::RParen) || self.check_type_token() {
                break;
            }

            // Handle attributes with type parameters: byval(type), sret(type), inalloca(type)
            if self.match_token(&Token::Byval) ||
               self.match_token(&Token::Sret) ||
               self.match_token(&Token::Inalloca) {
                // Check for optional type parameter
                if self.check(&Token::LParen) {
                    self.advance(); // consume (
                    // Skip tokens until we find )
                    while !self.check(&Token::RParen) && !self.is_at_end() {
                        self.advance();
                    }
                    self.match_token(&Token::RParen); // consume )
                }
                skip_count += 1;
                continue;
            }

            // Handle identifier-based attributes with type parameters: byref(type), elementtype(type), preallocated(type)
            if let Some(Token::Identifier(attr)) = self.peek() {
                if matches!(attr.as_str(), "byref" | "elementtype" | "preallocated") {
                    self.advance();
                    if self.check(&Token::LParen) {
                        self.advance(); // consume (
                        // Skip tokens until we find )
                        while !self.check(&Token::RParen) && !self.is_at_end() {
                            self.advance();
                        }
                        self.match_token(&Token::RParen); // consume )
                    }
                    skip_count += 1;
                    continue;
                }
                // Handle initializes with nested parentheses: initializes((0, 4))
                if attr == "initializes" {
                    self.advance();
                    if self.check(&Token::LParen) {
                        self.advance(); // consume first (
                        let mut depth = 1;
                        while depth > 0 && !self.is_at_end() {
                            if self.check(&Token::LParen) {
                                depth += 1;
                                self.advance();
                            } else if self.check(&Token::RParen) {
                                depth -= 1;
                                self.advance();
                            } else {
                                self.advance();
                            }
                        }
                    }
                    skip_count += 1;
                    continue;
                }
            }

            self.advance();
            skip_count += 1;
        }
    }

    fn skip_function_attributes(&mut self) {
        // Skip function attributes like noinline, alwaysinline, etc.
        while !self.is_at_end() && !self.check(&Token::LBrace) {
            if self.check(&Token::Hash) || self.check_attr_group_id() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_instruction_flags(&mut self) {
        loop {
            // Integer arithmetic flags (keyword tokens)
            if self.match_token(&Token::Nuw) ||
               self.match_token(&Token::Nsw) ||
               self.match_token(&Token::Exact) {
                continue;
            }

            // Fast-math flags (identifiers)
            if let Some(Token::Identifier(id)) = self.peek() {
                if matches!(id.as_str(), "fast" | "nnan" | "ninf" | "nsz" | "arcp" |
                                         "contract" | "afn" | "reassoc") {
                    self.advance();
                    continue;
                }
            }

            // No more flags
            break;
        }
    }

    fn skip_instruction_level_attributes(&mut self) {
        // Skip attributes that appear after instruction operands
        loop {
            // Attribute group IDs: #0, #1, etc.
            if self.check(&Token::Hash) || self.check_attr_group_id() {
                self.advance();
                continue;
            }

            // Keyword token attributes
            if self.match_token(&Token::Nounwind) ||
               self.match_token(&Token::Noreturn) {
                continue;
            }

            // Identifier-based attributes
            if let Some(Token::Identifier(attr)) = self.peek() {
                if matches!(attr.as_str(), "readonly" | "writeonly" | "readnone" |
                                           "nocapture" | "noinline" | "alwaysinline" |
                                           "cold" | "hot" | "convergent" | "speculatable") {
                    self.advance();
                    continue;
                }
            }

            // No more attributes
            break;
        }
    }

    fn skip_syncscope(&mut self) {
        // syncscope is a keyword token, not an identifier
        if self.match_token(&Token::Syncscope) {
            if self.check(&Token::LParen) {
                self.advance();
                while !self.check(&Token::RParen) && !self.is_at_end() {
                    self.advance();
                }
                self.consume(&Token::RParen).ok();
            }
        }
    }

    fn skip_memory_ordering(&mut self) -> bool {
        // Memory orderings are keyword tokens, not identifiers
        self.match_token(&Token::Unordered)
            || self.match_token(&Token::Monotonic)
            || self.match_token(&Token::Acquire)
            || self.match_token(&Token::Release)
            || self.match_token(&Token::Acq_rel)
            || self.match_token(&Token::Seq_cst)
    }

    fn skip_load_store_attributes(&mut self) {
        // Also handle attributes that appear without comma (syncscope, orderings)
        loop {
            // Check for syncscope("...") - using keyword token
            let before_pos = self.current;
            self.skip_syncscope();
            if self.current != before_pos {
                continue; // We consumed syncscope, continue loop
            }

            // Check for memory ordering keyword tokens
            if self.skip_memory_ordering() {
                continue;
            }

            // Check for comma-separated attributes
            if !self.match_token(&Token::Comma) {
                break;
            }

            if self.match_token(&Token::Align) {
                if let Some(Token::Integer(_)) = self.peek() {
                    self.advance();
                }
            } else if self.match_token(&Token::Volatile) {
                // consumed
            } else {
                self.current -= 1; // put back the comma
                break;
            }
        }
    }

    fn skip_until_newline_or_semicolon(&mut self) {
        // Skip metadata and other directives we don't parse yet
        let mut skip_count = 0;
        const MAX_SKIP: usize = 500;

        while !self.is_at_end() && skip_count < MAX_SKIP {
            let token = self.peek().unwrap();
            if matches!(token, Token::Define | Token::Declare | Token::Global | Token::Target | Token::Source_filename)
                || self.check_global_ident() {
                break;
            }
            self.advance();
            skip_count += 1;
        }
    }

    fn check_type_token(&self) -> bool {
        matches!(self.peek(), Some(Token::Void) | Some(Token::IntType(_)) |
                 Some(Token::Half) | Some(Token::Float) | Some(Token::Double) |
                 Some(Token::Ptr) | Some(Token::Label) | Some(Token::LBracket) |
                 Some(Token::LBrace) | Some(Token::LAngle))
    }

    fn check_local_ident(&self) -> bool {
        matches!(self.peek(), Some(Token::LocalIdent(_)))
    }

    fn check_global_ident(&self) -> bool {
        matches!(self.peek(), Some(Token::GlobalIdent(_)))
    }

    fn check_attr_group_id(&self) -> bool {
        matches!(self.peek(), Some(Token::AttrGroupId(_)))
    }

    fn peek_global_ident(&self) -> Option<String> {
        if let Some(Token::GlobalIdent(name)) = self.peek() {
            Some(name.clone())
        } else {
            None
        }
    }

    fn expect_global_ident(&mut self) -> ParseResult<String> {
        if let Some(Token::GlobalIdent(name)) = self.peek().cloned() {
            self.advance();
            Ok(name)
        } else {
            Err(ParseError::InvalidSyntax {
                message: "Expected global identifier".to_string(),
                position: self.current,
            })
        }
    }

    fn expect_local_ident(&mut self) -> ParseResult<String> {
        if let Some(Token::LocalIdent(name)) = self.peek().cloned() {
            self.advance();
            Ok(name)
        } else {
            Err(ParseError::InvalidSyntax {
                message: "Expected local identifier".to_string(),
                position: self.current,
            })
        }
    }

    // Token manipulation helpers

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn peek_ahead(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.current + n)
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens.get(self.current - 1)
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Some(Token::EOF) | None)
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

    fn is_metadata_token(&self) -> bool {
        // Check if current token is a metadata token
        // Either Token::Exclaim or Token::MetadataIdent
        if self.is_at_end() {
            return false;
        }
        matches!(self.peek(), Some(Token::Exclaim) | Some(Token::MetadataIdent(_)))
    }

    fn consume(&mut self, token: &Token) -> ParseResult<()> {
        if self.check(token) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", token),
                found: format!("{:?}", self.peek()),
                position: self.current,
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
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let module = result.unwrap();
        assert_eq!(module.function_count(), 1);
    }

    #[test]
    fn test_parse_function_with_return() {
        let ctx = Context::new();
        let source = r#"
            define i32 @foo() {
                ret i32 42
            }
        "#;

        let result = parse(source, ctx);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
    }
}
