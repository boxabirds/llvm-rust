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
        // declare [ret attrs] type @name([params])
        self.skip_attributes();

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
        // define [linkage] [ret attrs] type @name([params]) [fn attrs] { body }
        self.skip_linkage_and_visibility();
        self.skip_attributes();

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
                    // Common keywords that can be used as labels
                    Token::Entry => Some("entry".to_string()),
                    Token::Cleanup => Some("cleanup".to_string()),
                    Token::Catch => Some("catch".to_string()),
                    Token::Filter => Some("filter".to_string()),
                    // Any other token followed by colon is not a valid label
                    _ => None,
                };

                if let Some(name) = label_name {
                    self.advance(); // consume label token
                    self.advance(); // consume colon
                    Some(name)
                } else {
                    // Not a recognized label token
                    None
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

            // Check if next token is a label (local ident followed by colon)
            if let Some(Token::LocalIdent(_)) = self.peek() {
                if self.peek_ahead(1) == Some(&Token::Colon) {
                    break;
                }
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
            Opcode::Call => {
                // call type @func(args...)
                let _ret_ty = self.parse_type()?;
                let _func = self.parse_value()?;
                self.consume(&Token::LParen)?;
                let _args = self.parse_call_arguments()?;
                self.consume(&Token::RParen)?;
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
                // alloca type [, align ...]
                let _ty = self.parse_type()?;
                // Skip alignment and other attributes
                while self.match_token(&Token::Comma) {
                    if self.match_token(&Token::Align) {
                        if let Some(Token::Integer(_)) = self.peek() {
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
            }
            Opcode::Load => {
                // load type, ptr %ptr [, align ...]
                let _ty = self.parse_type()?;
                self.consume(&Token::Comma)?;
                let _ptr_ty = self.parse_type()?;
                let _ptr = self.parse_value()?;
                // Skip attributes
                self.skip_load_store_attributes();
            }
            Opcode::Store => {
                // store type %val, ptr %ptr [, align ...]
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
                    if self.check_local_ident() && self.peek_ahead(1) == Some(&Token::Colon) {
                        // Label
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

    fn parse_call_arguments(&mut self) -> ParseResult<Vec<(Type, Value)>> {
        let mut args = Vec::new();

        while !self.check(&Token::RParen) && !self.is_at_end() {
            let ty = self.parse_type()?;
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

        match token {
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
                // Modern LLVM uses opaque pointers (ptr)
                Ok(self.context.ptr_type(self.context.int8_type()))
            }
            Token::Label => {
                self.advance();
                Ok(self.context.label_type())
            }
            Token::Metadata => {
                self.advance();
                Ok(self.context.metadata_type())
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
                // Vector type: < size x type >
                self.advance();
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
        }
    }

    fn parse_value(&mut self) -> ParseResult<Value> {
        let token = self.peek().ok_or(ParseError::UnexpectedEOF)?;

        match token {
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
            Token::Undef => {
                self.advance();
                Ok(Value::undef(self.context.void_type()))
            }
            Token::Zeroinitializer => {
                self.advance();
                Ok(Value::zero_initializer(self.context.void_type()))
            }
            _ => {
                Err(ParseError::InvalidSyntax {
                    message: format!("Expected value, found {:?}", token),
                    position: self.current,
                })
            }
        }
    }

    fn parse_parameters(&mut self) -> ParseResult<Vec<(Type, String)>> {
        let mut params = Vec::new();

        while !self.check(&Token::RParen) && !self.is_at_end() {
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
        while self.match_token(&Token::Private) ||
              self.match_token(&Token::Internal) ||
              self.match_token(&Token::External) ||
              self.match_token(&Token::Weak) ||
              self.match_token(&Token::Linkonce) ||
              self.match_token(&Token::Common) ||
              self.match_token(&Token::Appending) ||
              self.match_token(&Token::Hidden) ||
              self.match_token(&Token::Protected) ||
              self.match_token(&Token::Default) ||
              self.match_token(&Token::Dllimport) ||
              self.match_token(&Token::Dllexport) ||
              self.match_token(&Token::Unnamed_addr) {
            // Keep consuming
        }
    }

    fn skip_attributes(&mut self) {
        while self.check(&Token::Hash) || self.check_attr_group_id() {
            self.advance();
        }
    }

    fn skip_parameter_attributes(&mut self) {
        // Skip attributes like readonly, nonnull, etc.
        let mut skip_count = 0;
        const MAX_ATTR_SKIP: usize = 50;

        while !self.is_at_end() && skip_count < MAX_ATTR_SKIP {
            if self.check_local_ident() || self.check(&Token::Comma) ||
               self.check(&Token::RParen) || self.check_type_token() {
                break;
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
        while self.match_token(&Token::Nuw) ||
              self.match_token(&Token::Nsw) ||
              self.match_token(&Token::Exact) {
            // Continue
        }
    }

    fn skip_load_store_attributes(&mut self) {
        while self.match_token(&Token::Comma) {
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
