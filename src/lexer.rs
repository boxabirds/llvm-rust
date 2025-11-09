//! LLVM IR Lexer
//!
//! Tokenizes LLVM IR text into a stream of tokens for parsing.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Define,
    Declare,
    Global,
    Constant,
    External,
    Private,
    Internal,
    Available_externally,
    Linkonce,
    Weak,
    Common,
    Appending,
    Extern_weak,
    Linkonce_odr,
    Weak_odr,
    Dllimport,
    Dllexport,
    Hidden,
    Protected,
    Default,
    Thread_local,
    Unnamed_addr,
    Local_unnamed_addr,
    Type,
    Opaque,
    Null,
    True,
    False,
    Zeroinitializer,
    Undef,
    Poison,
    To,
    Nuw,
    Nsw,
    Exact,
    Inbounds,
    Volatile,
    Atomic,
    Unordered,
    Monotonic,
    Acquire,
    Release,
    Acq_rel,
    Seq_cst,
    Singlethread,
    Target,
    Datalayout,
    Triple,
    Source_filename,
    Attributes,
    Align,
    Addrspace,
    Section,
    Comdat,
    Gc,
    Prefix,
    Prologue,
    Personality,
    Alias,
    Ifunc,
    Entry,
    Distinct,
    Nounwind,
    Inreg,
    Byval,
    Inalloca,
    Sret,
    Noalias,
    Nocapture,
    Nest,
    Returned,
    Nonnull,
    Dereferenceable,
    Dereferenceable_or_null,
    Swiftself,
    Swifterror,
    Immarg,
    Zeroext,
    Signext,
    Inlinehint,
    Alwaysinline,
    Noinline,
    Optsize,
    Optnone,
    Minsize,
    Noreturn,
    Norecurse,
    Willreturn,
    Nosync,
    Sanitize_address,
    Sanitize_thread,
    Sanitize_memory,
    Sanitize_hwaddress,
    Safestack,
    Uwtable,
    Nocf_check,
    Shadowcallstack,
    Mustprogress,
    Vscale_range,
    Strictfp,
    Naked,
    Builtin,
    Cold,
    Hot,
    Nobuiltin,
    Noduplicate,
    Noimplicitfloat,
    Nomerge,
    Nonlazybind,
    Noredzone,
    Null_pointer_is_valid,
    Optforfuzzing,
    Readnone,
    Readonly,
    Writeonly,
    Argmemonly,
    Inaccessiblememonly,
    Inaccessiblemem_or_argmemonly,
    Speculatable,
    Returns_twice,
    Ssp,
    Sspreq,
    Sspstrong,
    Thunk,
    Amdgpu_kernel,
    Amdgpu_cs_chain,
    Amdgpu_ps,
    Syncscope,
    Var,
    Dso_local,
    Dso_preemptable,
    Filename,
    Name,

    // Instructions
    Ret,
    Br,
    Switch,
    IndirectBr,
    Invoke,
    Resume,
    Unreachable,
    CleanupRet,
    CatchRet,
    CatchSwitch,
    CallBr,
    FNeg,
    Add,
    FAdd,
    Sub,
    FSub,
    Mul,
    FMul,
    UDiv,
    SDiv,
    FDiv,
    URem,
    SRem,
    FRem,
    Shl,
    LShr,
    AShr,
    And,
    Or,
    Xor,
    ExtractElement,
    InsertElement,
    ShuffleVector,
    ExtractValue,
    InsertValue,
    Alloca,
    Load,
    Store,
    GetElementPtr,
    Fence,
    AtomicCmpXchg,
    AtomicRMW,
    Trunc,
    ZExt,
    SExt,
    FPToUI,
    FPToSI,
    UIToFP,
    SIToFP,
    FPTrunc,
    FPExt,
    PtrToInt,
    IntToPtr,
    BitCast,
    AddrSpaceCast,
    ICmp,
    FCmp,
    Phi,
    Call,
    Select,
    VAArg,
    LandingPad,
    Cleanup,
    Catch,
    Filter,

    // Comparison predicates
    Eq,
    Ne,
    Ugt,
    Uge,
    Ult,
    Ule,
    Sgt,
    Sge,
    Slt,
    Sle,
    Oeq,
    Ogt,
    Oge,
    Olt,
    Ole,
    One,
    Ord,
    Uno,
    Ueq,

    // Types
    Void,
    Half,
    Bfloat,
    Float,
    Double,
    X86_fp80,
    Fp128,
    Ppc_fp128,
    X86_mmx,
    X86_amx,
    Ptr,
    Label,
    Token,
    Metadata,
    IntType(u32),        // i1, i8, i16, i32, i64, i128, etc.
    X,                   // 'x' in array/vector types like [4 x i8]

    // Literals and identifiers
    LocalIdent(String),  // %name or %0
    GlobalIdent(String), // @name or @0
    MetadataIdent(String), // !name or !0
    AttrGroupId(u32),    // #0, #1, etc.
    Identifier(String),  // Bare identifiers (for labels like BB1, then, etc.)
    Integer(i128),
    Float64(f64),
    StringLit(String),
    CString(Vec<u8>),

    // Symbols
    Equal,
    Comma,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LAngle,
    RAngle,
    Star,
    Colon,
    Ellipsis,
    Pipe,
    Exclaim,
    Hash,

    // Special
    EOF,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::LocalIdent(s) => write!(f, "%{}", s),
            Token::GlobalIdent(s) => write!(f, "@{}", s),
            Token::MetadataIdent(s) => write!(f, "!{}", s),
            Token::IntType(bits) => write!(f, "i{}", bits),
            Token::Integer(n) => write!(f, "{}", n),
            Token::Float64(n) => write!(f, "{}", n),
            Token::StringLit(s) => write!(f, "\"{}\"", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_and_comments();

            if self.is_at_end() {
                tokens.push(Token::EOF);
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, String> {
        let ch = self.current_char();

        match ch {
            '=' => { self.advance(); Ok(Token::Equal) }
            ',' => { self.advance(); Ok(Token::Comma) }
            '(' => { self.advance(); Ok(Token::LParen) }
            ')' => { self.advance(); Ok(Token::RParen) }
            '{' => { self.advance(); Ok(Token::LBrace) }
            '}' => { self.advance(); Ok(Token::RBrace) }
            '[' => { self.advance(); Ok(Token::LBracket) }
            ']' => { self.advance(); Ok(Token::RBracket) }
            '<' => { self.advance(); Ok(Token::LAngle) }
            '>' => { self.advance(); Ok(Token::RAngle) }
            '*' => { self.advance(); Ok(Token::Star) }
            ':' => { self.advance(); Ok(Token::Colon) }
            '|' => { self.advance(); Ok(Token::Pipe) }
            '!' => {
                self.advance();
                if self.current_char().is_ascii_alphanumeric() || self.current_char() == '_' {
                    self.read_metadata_ident()
                } else {
                    Ok(Token::Exclaim)
                }
            }
            '^' => {
                // Summary/module metadata reference (e.g., ^0, ^1)
                self.advance();
                if self.current_char().is_ascii_digit() {
                    // Read numeric metadata reference
                    self.read_metadata_ident()
                } else {
                    // Just return caret as a token (for future use)
                    Ok(Token::Exclaim) // Treat as metadata marker for now
                }
            }
            '#' => {
                self.advance();
                if self.current_char().is_ascii_digit() {
                    let num = self.read_number_literal()?;
                    if let Token::Integer(n) = num {
                        Ok(Token::AttrGroupId(n as u32))
                    } else {
                        Err(format!("Expected integer after #"))
                    }
                } else {
                    Ok(Token::Hash)
                }
            }
            '%' => {
                self.advance();
                self.read_local_ident()
            }
            '@' => {
                self.advance();
                self.read_global_ident()
            }
            '"' => self.read_string_literal(),
            'c' if self.peek_char() == Some('"') => self.read_c_string(),
            '-' | '0'..='9' => self.read_number_literal(),
            'a'..='z' | 'A'..='Z' | '_' => self.read_keyword_or_ident(),
            '.' if self.peek_char() == Some('.') && self.peek_ahead(2) == Some('.') => {
                self.advance();
                self.advance();
                self.advance();
                Ok(Token::Ellipsis)
            }
            _ => Err(format!("Unexpected character '{}' at line {}, column {}", ch, self.line, self.column))
        }
    }

    fn read_local_ident(&mut self) -> Result<Token, String> {
        let mut name = String::new();

        // Handle numeric identifiers like %0, %1
        if self.current_char().is_ascii_digit() {
            while !self.is_at_end() && self.current_char().is_ascii_digit() {
                name.push(self.current_char());
                self.advance();
            }
            return Ok(Token::LocalIdent(name));
        }

        // Handle quoted identifiers
        if self.current_char() == '"' {
            self.advance(); // skip opening "
            while !self.is_at_end() && self.current_char() != '"' {
                if self.current_char() == '\\' {
                    self.advance();
                    if !self.is_at_end() {
                        name.push(self.current_char());
                        self.advance();
                    }
                } else {
                    name.push(self.current_char());
                    self.advance();
                }
            }
            if self.current_char() == '"' {
                self.advance(); // skip closing "
            }
            return Ok(Token::LocalIdent(name));
        }

        // Handle normal identifiers
        while !self.is_at_end() {
            let ch = self.current_char();
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' || ch == '-' {
                name.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        Ok(Token::LocalIdent(name))
    }

    fn read_global_ident(&mut self) -> Result<Token, String> {
        let mut name = String::new();

        // Handle numeric identifiers
        if self.current_char().is_ascii_digit() {
            while !self.is_at_end() && self.current_char().is_ascii_digit() {
                name.push(self.current_char());
                self.advance();
            }
            return Ok(Token::GlobalIdent(name));
        }

        // Handle quoted identifiers
        if self.current_char() == '"' {
            self.advance(); // skip opening "
            while !self.is_at_end() && self.current_char() != '"' {
                if self.current_char() == '\\' {
                    self.advance();
                    if !self.is_at_end() {
                        name.push(self.current_char());
                        self.advance();
                    }
                } else {
                    name.push(self.current_char());
                    self.advance();
                }
            }
            if self.current_char() == '"' {
                self.advance(); // skip closing "
            }
            return Ok(Token::GlobalIdent(name));
        }

        // Handle normal identifiers
        while !self.is_at_end() {
            let ch = self.current_char();
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' || ch == '-' || ch == '$' {
                name.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        Ok(Token::GlobalIdent(name))
    }

    fn read_metadata_ident(&mut self) -> Result<Token, String> {
        let mut name = String::new();

        while !self.is_at_end() {
            let ch = self.current_char();
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' || ch == '-' {
                name.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        Ok(Token::MetadataIdent(name))
    }

    fn read_string_literal(&mut self) -> Result<Token, String> {
        self.advance(); // skip opening "
        let mut s = String::new();

        while !self.is_at_end() && self.current_char() != '"' {
            if self.current_char() == '\\' {
                self.advance();
                if !self.is_at_end() {
                    let escape = self.current_char();
                    match escape {
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        '0' => s.push('\0'),
                        _ => s.push(escape),
                    }
                    self.advance();
                }
            } else {
                s.push(self.current_char());
                self.advance();
            }
        }

        if self.current_char() == '"' {
            self.advance(); // skip closing "
        }

        Ok(Token::StringLit(s))
    }

    fn read_c_string(&mut self) -> Result<Token, String> {
        self.advance(); // skip 'c'
        self.advance(); // skip '"'
        let mut bytes = Vec::new();

        while !self.is_at_end() && self.current_char() != '"' {
            if self.current_char() == '\\' {
                self.advance();
                if !self.is_at_end() {
                    let escape = self.current_char();
                    // Parse hex escape sequences like \FF
                    if escape.is_ascii_hexdigit() {
                        let mut hex = String::new();
                        hex.push(escape);
                        self.advance();
                        if !self.is_at_end() && self.current_char().is_ascii_hexdigit() {
                            hex.push(self.current_char());
                            self.advance();
                        }
                        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                            bytes.push(byte);
                        }
                    } else {
                        match escape {
                            'n' => bytes.push(b'\n'),
                            'r' => bytes.push(b'\r'),
                            't' => bytes.push(b'\t'),
                            '\\' => bytes.push(b'\\'),
                            '"' => bytes.push(b'"'),
                            '0' => bytes.push(0),
                            _ => bytes.push(escape as u8),
                        }
                        self.advance();
                    }
                }
            } else {
                bytes.push(self.current_char() as u8);
                self.advance();
            }
        }

        if self.current_char() == '"' {
            self.advance(); // skip closing "
        }

        Ok(Token::CString(bytes))
    }

    fn read_number_literal(&mut self) -> Result<Token, String> {
        let mut num = String::new();
        let is_negative = self.current_char() == '-';

        if is_negative {
            num.push('-');
            self.advance();
        }

        // Check for hex number (0x...)
        if self.current_char() == '0' && self.peek_char() == Some('x') {
            num.push('0');
            self.advance();
            num.push('x');
            self.advance();

            while !self.is_at_end() && (self.current_char().is_ascii_hexdigit() || self.current_char() == '_') {
                if self.current_char() != '_' {
                    num.push(self.current_char());
                }
                self.advance();
            }

            let value = i128::from_str_radix(&num[2..], 16)
                .map_err(|e| format!("Invalid hex number: {}", e))?;
            return Ok(Token::Integer(if is_negative { -value } else { value }));
        }

        // Read integer part
        while !self.is_at_end() && (self.current_char().is_ascii_digit() || self.current_char() == '_') {
            if self.current_char() != '_' {
                num.push(self.current_char());
            }
            self.advance();
        }

        // Check for float (has decimal point or exponent)
        if !self.is_at_end() && (self.current_char() == '.' || self.current_char() == 'e' || self.current_char() == 'E') {
            if self.current_char() == '.' {
                num.push('.');
                self.advance();

                while !self.is_at_end() && self.current_char().is_ascii_digit() {
                    num.push(self.current_char());
                    self.advance();
                }
            }

            if !self.is_at_end() && (self.current_char() == 'e' || self.current_char() == 'E') {
                num.push(self.current_char());
                self.advance();

                if !self.is_at_end() && (self.current_char() == '+' || self.current_char() == '-') {
                    num.push(self.current_char());
                    self.advance();
                }

                while !self.is_at_end() && self.current_char().is_ascii_digit() {
                    num.push(self.current_char());
                    self.advance();
                }
            }

            let value = num.parse::<f64>()
                .map_err(|e| format!("Invalid float number: {}", e))?;
            return Ok(Token::Float64(value));
        }

        // It's an integer
        let value = num.parse::<i128>()
            .map_err(|e| format!("Invalid integer: {}", e))?;
        Ok(Token::Integer(value))
    }

    fn read_keyword_or_ident(&mut self) -> Result<Token, String> {
        let mut word = String::new();

        while !self.is_at_end() {
            let ch = self.current_char();
            // Allow dots and hyphens in identifiers (e.g., labels like "then.7", "no_exit.2")
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' || ch == '-' {
                word.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check if it's an integer type
        if word.starts_with('i') && word.len() > 1 {
            if let Ok(bits) = word[1..].parse::<u32>() {
                return Ok(Token::IntType(bits));
            }
        }

        // Check for keywords
        let token = match word.as_str() {
            "define" => Token::Define,
            "declare" => Token::Declare,
            "global" => Token::Global,
            "constant" => Token::Constant,
            "external" => Token::External,
            "private" => Token::Private,
            "internal" => Token::Internal,
            "available_externally" => Token::Available_externally,
            "linkonce" => Token::Linkonce,
            "weak" => Token::Weak,
            "common" => Token::Common,
            "appending" => Token::Appending,
            "extern_weak" => Token::Extern_weak,
            "linkonce_odr" => Token::Linkonce_odr,
            "weak_odr" => Token::Weak_odr,
            "dllimport" => Token::Dllimport,
            "dllexport" => Token::Dllexport,
            "hidden" => Token::Hidden,
            "protected" => Token::Protected,
            "default" => Token::Default,
            "thread_local" => Token::Thread_local,
            "unnamed_addr" => Token::Unnamed_addr,
            "local_unnamed_addr" => Token::Local_unnamed_addr,
            "type" => Token::Type,
            "opaque" => Token::Opaque,
            "null" => Token::Null,
            "true" => Token::True,
            "false" => Token::False,
            "zeroinitializer" => Token::Zeroinitializer,
            "undef" => Token::Undef,
            "poison" => Token::Poison,
            "to" => Token::To,
            "nuw" => Token::Nuw,
            "nsw" => Token::Nsw,
            "exact" => Token::Exact,
            "inbounds" => Token::Inbounds,
            "volatile" => Token::Volatile,
            "atomic" => Token::Atomic,
            "unordered" => Token::Unordered,
            "monotonic" => Token::Monotonic,
            "acquire" => Token::Acquire,
            "release" => Token::Release,
            "acq_rel" => Token::Acq_rel,
            "seq_cst" => Token::Seq_cst,
            "singlethread" => Token::Singlethread,
            "target" => Token::Target,
            "datalayout" => Token::Datalayout,
            "triple" => Token::Triple,
            "source_filename" => Token::Source_filename,
            "attributes" => Token::Attributes,
            "align" => Token::Align,
            "addrspace" => Token::Addrspace,
            "section" => Token::Section,
            "comdat" => Token::Comdat,
            "gc" => Token::Gc,
            "prefix" => Token::Prefix,
            "prologue" => Token::Prologue,
            "personality" => Token::Personality,

            // Instructions
            "ret" => Token::Ret,
            "br" => Token::Br,
            "switch" => Token::Switch,
            "indirectbr" => Token::IndirectBr,
            "invoke" => Token::Invoke,
            "resume" => Token::Resume,
            "unreachable" => Token::Unreachable,
            "cleanupret" => Token::CleanupRet,
            "catchret" => Token::CatchRet,
            "catchswitch" => Token::CatchSwitch,
            "callbr" => Token::CallBr,
            "fneg" => Token::FNeg,
            "add" => Token::Add,
            "fadd" => Token::FAdd,
            "sub" => Token::Sub,
            "fsub" => Token::FSub,
            "mul" => Token::Mul,
            "fmul" => Token::FMul,
            "udiv" => Token::UDiv,
            "sdiv" => Token::SDiv,
            "fdiv" => Token::FDiv,
            "urem" => Token::URem,
            "srem" => Token::SRem,
            "frem" => Token::FRem,
            "shl" => Token::Shl,
            "lshr" => Token::LShr,
            "ashr" => Token::AShr,
            "and" => Token::And,
            "or" => Token::Or,
            "xor" => Token::Xor,
            "extractelement" => Token::ExtractElement,
            "insertelement" => Token::InsertElement,
            "shufflevector" => Token::ShuffleVector,
            "extractvalue" => Token::ExtractValue,
            "insertvalue" => Token::InsertValue,
            "alloca" => Token::Alloca,
            "load" => Token::Load,
            "store" => Token::Store,
            "getelementptr" => Token::GetElementPtr,
            "fence" => Token::Fence,
            "cmpxchg" => Token::AtomicCmpXchg,
            "atomicrmw" => Token::AtomicRMW,
            "trunc" => Token::Trunc,
            "zext" => Token::ZExt,
            "sext" => Token::SExt,
            "fptoui" => Token::FPToUI,
            "fptosi" => Token::FPToSI,
            "uitofp" => Token::UIToFP,
            "sitofp" => Token::SIToFP,
            "fptrunc" => Token::FPTrunc,
            "fpext" => Token::FPExt,
            "ptrtoint" => Token::PtrToInt,
            "inttoptr" => Token::IntToPtr,
            "bitcast" => Token::BitCast,
            "addrspacecast" => Token::AddrSpaceCast,
            "icmp" => Token::ICmp,
            "fcmp" => Token::FCmp,
            "phi" => Token::Phi,
            "call" => Token::Call,
            "select" => Token::Select,
            "va_arg" => Token::VAArg,
            "landingpad" => Token::LandingPad,
            "cleanup" => Token::Cleanup,
            "catch" => Token::Catch,
            "filter" => Token::Filter,

            // Comparison predicates
            "eq" => Token::Eq,
            "ne" => Token::Ne,
            "ugt" => Token::Ugt,
            "uge" => Token::Uge,
            "ult" => Token::Ult,
            "ule" => Token::Ule,
            "sgt" => Token::Sgt,
            "sge" => Token::Sge,
            "slt" => Token::Slt,
            "sle" => Token::Sle,
            "oeq" => Token::Oeq,
            "ogt" => Token::Ogt,
            "oge" => Token::Oge,
            "olt" => Token::Olt,
            "ole" => Token::Ole,
            "one" => Token::One,
            "ord" => Token::Ord,
            "uno" => Token::Uno,
            "ueq" => Token::Ueq,

            // Types
            "void" => Token::Void,
            "half" => Token::Half,
            "bfloat" => Token::Bfloat,
            "float" => Token::Float,
            "double" => Token::Double,
            "x86_fp80" => Token::X86_fp80,
            "fp128" => Token::Fp128,
            "ppc_fp128" => Token::Ppc_fp128,
            "x86_mmx" => Token::X86_mmx,
            "x86_amx" => Token::X86_amx,
            "ptr" => Token::Ptr,
            "label" => Token::Label,
            "token" => Token::Token,
            "metadata" => Token::Metadata,
            "x" => Token::X,
            "alias" => Token::Alias,
            "ifunc" => Token::Ifunc,
            "entry" => Token::Entry,
            "distinct" => Token::Distinct,
            "nounwind" => Token::Nounwind,
            "inreg" => Token::Inreg,
            "byval" => Token::Byval,
            "inalloca" => Token::Inalloca,
            "sret" => Token::Sret,
            "noalias" => Token::Noalias,
            "nocapture" => Token::Nocapture,
            "nest" => Token::Nest,
            "returned" => Token::Returned,
            "nonnull" => Token::Nonnull,
            "dereferenceable" => Token::Dereferenceable,
            "dereferenceable_or_null" => Token::Dereferenceable_or_null,
            "swiftself" => Token::Swiftself,
            "swifterror" => Token::Swifterror,
            "immarg" => Token::Immarg,
            "zeroext" => Token::Zeroext,
            "signext" => Token::Signext,
            "inlinehint" => Token::Inlinehint,
            "alwaysinline" => Token::Alwaysinline,
            "noinline" => Token::Noinline,
            "optsize" => Token::Optsize,
            "optnone" => Token::Optnone,
            "minsize" => Token::Minsize,
            "noreturn" => Token::Noreturn,
            "norecurse" => Token::Norecurse,
            "willreturn" => Token::Willreturn,
            "nosync" => Token::Nosync,
            "sanitize_address" => Token::Sanitize_address,
            "sanitize_thread" => Token::Sanitize_thread,
            "sanitize_memory" => Token::Sanitize_memory,
            "sanitize_hwaddress" => Token::Sanitize_hwaddress,
            "safestack" => Token::Safestack,
            "uwtable" => Token::Uwtable,
            "nocf_check" => Token::Nocf_check,
            "shadowcallstack" => Token::Shadowcallstack,
            "mustprogress" => Token::Mustprogress,
            "vscale_range" => Token::Vscale_range,
            "strictfp" => Token::Strictfp,
            "naked" => Token::Naked,
            "builtin" => Token::Builtin,
            "cold" => Token::Cold,
            "hot" => Token::Hot,
            "nobuiltin" => Token::Nobuiltin,
            "noduplicate" => Token::Noduplicate,
            "noimplicitfloat" => Token::Noimplicitfloat,
            "nomerge" => Token::Nomerge,
            "nonlazybind" => Token::Nonlazybind,
            "noredzone" => Token::Noredzone,
            "null_pointer_is_valid" => Token::Null_pointer_is_valid,
            "optforfuzzing" => Token::Optforfuzzing,
            "readnone" => Token::Readnone,
            "readonly" => Token::Readonly,
            "writeonly" => Token::Writeonly,
            "argmemonly" => Token::Argmemonly,
            "inaccessiblememonly" => Token::Inaccessiblememonly,
            "inaccessiblemem_or_argmemonly" => Token::Inaccessiblemem_or_argmemonly,
            "speculatable" => Token::Speculatable,
            "returns_twice" => Token::Returns_twice,
            "ssp" => Token::Ssp,
            "sspreq" => Token::Sspreq,
            "sspstrong" => Token::Sspstrong,
            "thunk" => Token::Thunk,
            "amdgpu_kernel" => Token::Amdgpu_kernel,
            "amdgpu_cs_chain" => Token::Amdgpu_cs_chain,
            "amdgpu_ps" => Token::Amdgpu_ps,
            "syncscope" => Token::Syncscope,
            "var" => Token::Var,
            "dso_local" => Token::Dso_local,
            "dso_preemptable" => Token::Dso_preemptable,
            "filename" => Token::Filename,
            "name" => Token::Name,

            _ => {
                // Unknown keyword - return as bare identifier (used for labels like BB1, then, etc.)
                Token::Identifier(word)
            }
        };

        Ok(token)
    }

    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_at_end() {
            match self.current_char() {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                ';' => {
                    // Skip comment until end of line
                    while !self.is_at_end() && self.current_char() != '\n' {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn current_char(&self) -> char {
        if self.position < self.input.len() {
            self.input[self.position]
        } else {
            '\0'
        }
    }

    fn peek_char(&self) -> Option<char> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }

    fn peek_ahead(&self, n: usize) -> Option<char> {
        if self.position + n < self.input.len() {
            Some(self.input[self.position + n])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        if self.position < self.input.len() {
            self.position += 1;
            self.column += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut lexer = Lexer::new("define void @main() { ret void }");
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0], Token::Define));
        assert!(matches!(tokens[1], Token::Void));
        assert!(matches!(tokens[2], Token::GlobalIdent(_)));
        assert!(matches!(tokens[3], Token::LParen));
        assert!(matches!(tokens[4], Token::RParen));
    }

    #[test]
    fn test_integer_types() {
        let mut lexer = Lexer::new("i1 i8 i16 i32 i64 i128");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::IntType(1));
        assert_eq!(tokens[1], Token::IntType(8));
        assert_eq!(tokens[2], Token::IntType(16));
        assert_eq!(tokens[3], Token::IntType(32));
        assert_eq!(tokens[4], Token::IntType(64));
        assert_eq!(tokens[5], Token::IntType(128));
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 -100 3.14 -2.5e10");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Integer(42));
        assert_eq!(tokens[1], Token::Integer(-100));
        assert!(matches!(tokens[2], Token::Float64(_)));
        assert!(matches!(tokens[3], Token::Float64(_)));
    }
}
