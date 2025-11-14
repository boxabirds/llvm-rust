//! LLVM IR Parser
//!
//! This module provides functionality to parse LLVM IR from text format (.ll files).

use crate::lexer::{Lexer, Token};
use crate::module::{Module, GlobalVariable};
use crate::function::{Function, CallingConvention};
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
    /// Invalid attribute
    InvalidAttribute { message: String },
}

/// Parse result
pub type ParseResult<T> = Result<T, ParseError>;

/// LLVM IR Parser
pub struct Parser {
    context: Context,
    tokens: Vec<Token>,
    current: usize,
    /// Symbol table for tracking local values within a function
    symbol_table: std::collections::HashMap<String, Value>,
    /// Function declarations table for tracking global function types
    function_decls: std::collections::HashMap<String, Type>,
    /// Type table for tracking numbered type definitions (%0, %1, etc.)
    type_table: std::collections::HashMap<String, Type>,
    /// Metadata registry for numbered metadata nodes (!0, !1, etc.)
    metadata_registry: std::collections::HashMap<String, crate::metadata::Metadata>,
    /// Attribute groups registry for #0, #1, etc.
    attribute_groups: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
}

impl Parser {
    pub fn new(context: Context) -> Self {
        Self {
            context,
            tokens: Vec::new(),
            current: 0,
            symbol_table: std::collections::HashMap::new(),
            function_decls: std::collections::HashMap::new(),
            type_table: std::collections::HashMap::new(),
            metadata_registry: std::collections::HashMap::new(),
            attribute_groups: std::collections::HashMap::new(),
        }
    }

    /// Parse a module from source code
    pub fn parse_module(&mut self, source: &str) -> ParseResult<Module> {
        // Tokenize
        let mut lexer = Lexer::new(source);
        self.tokens = lexer.tokenize().map_err(ParseError::LexerError)?;
        self.current = 0;

        // Pre-scan and parse type declarations in multiple passes to handle forward references
        self.parse_all_type_declarations()?;
        self.current = 0; // Reset to beginning

        let module = Module::new("parsed_module".to_string(), self.context.clone());

        // Parse module contents with safety limit to prevent infinite loops
        let mut iterations = 0;
        const MAX_MODULE_ITERATIONS: usize = 100000;

        while !self.is_at_end() && iterations < MAX_MODULE_ITERATIONS {
            iterations += 1;
            // Note: Attribute group parsing moved to a dedicated section below
            // (not skipping attributes anymore)

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

            // Parse type declarations: %TypeName = type { ... } or %0 = type { ... }
            if (self.peek_global_ident().is_some() || self.check_local_ident())
                && self.peek_ahead(1) == Some(&Token::Equal)
                && self.peek_ahead(2) == Some(&Token::Type) {
                self.parse_type_declaration()?;
                continue;
            }

            // Skip comdat definitions: $name = comdat any/exactmatch/largest/...
            if self.peek_global_ident().is_some() && self.peek_ahead(1) == Some(&Token::Equal)
                && self.peek_ahead(2) == Some(&Token::Comdat) {
                self.advance(); // skip $name
                self.advance(); // skip =
                self.advance(); // skip comdat
                if let Some(Token::Identifier(_)) = self.peek() {
                    self.advance(); // skip comdat type (any, exactmatch, etc.)
                }
                continue;
            }

            // Parse global variables: @name = [linkage] [externally_initialized] global/constant
            if self.peek_global_ident().is_some() && self.peek_ahead(1) == Some(&Token::Equal) {
                // Look ahead to find if this is a global variable (has 'global' or 'constant' keyword)
                let mut is_global_var = false;
                for offset in 2..10 {  // Check up to 10 tokens ahead for global/constant
                    if let Some(tok) = self.peek_ahead(offset) {
                        if matches!(tok, Token::Global | Token::Constant) {
                            is_global_var = true;
                            break;
                        }
                        // Stop if we hit a token that definitely means it's not a global var
                        if matches!(tok, Token::Alias | Token::Ifunc | Token::Define | Token::Declare | Token::Comdat) {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if is_global_var {
                    let global = self.parse_global_variable()?;
                    module.add_global(global);
                    continue;
                }
            }

            // Parse alias declarations: @name = [linkage] alias type, aliasee
            if self.peek_global_ident().is_some() && self.peek_ahead(1) == Some(&Token::Equal) {
                // Check if it's an alias by looking ahead
                let mut idx = 2;
                // Skip linkage/visibility keywords
                while let Some(tok) = self.peek_ahead(idx) {
                    if matches!(tok, Token::Identifier(_)) {
                        idx += 1;
                        continue;
                    }
                    if matches!(tok, Token::Alias) {
                        // Parse the alias
                        let alias = self.parse_alias()?;
                        module.add_alias(alias);
                        break;
                    }
                    if matches!(tok, Token::Ifunc) {
                        // Skip ifunc declarations for now
                        self.advance(); // skip @name
                        self.advance(); // skip =
                        while !self.is_at_end() && !self.check(&Token::Define) && !self.check(&Token::Declare) &&
                              !self.peek_global_ident().is_some() {
                            self.advance();
                        }
                        break;
                    }
                    break;
                }
            }

            // Parse function declarations
            if self.match_token(&Token::Declare) {
                let function = self.parse_function_declaration()?;
                // Track function type for call validation
                self.function_decls.insert(function.name().to_string(), function.get_type());
                module.add_function(function);
                continue;
            }

            // Parse function definitions
            if self.match_token(&Token::Define) {
                let function = self.parse_function_definition()?;
                // Track function type for call validation
                self.function_decls.insert(function.name().to_string(), function.get_type());
                module.add_function(function);
                continue;
            }

            // Parse attribute group definitions: attributes #0 = { ... }
            if self.match_token(&Token::Attributes) {
                // Get attribute group ID (#0, #1, etc.)
                let mut group_id = String::new();
                if let Some(Token::AttrGroupId(num)) = self.peek() {
                    group_id = format!("#{}", num);
                    self.advance();
                }
                // Consume '='
                self.match_token(&Token::Equal);
                // Parse attribute list in braces
                let mut attrs = std::collections::HashMap::new();
                if self.match_token(&Token::LBrace) {
                    while !self.check(&Token::RBrace) && !self.is_at_end() {
                        // Parse string attributes: "key"="value" or "key"
                        if let Some(Token::StringLit(key)) = self.peek().cloned() {
                            self.advance();
                            if self.match_token(&Token::Equal) {
                                if let Some(Token::StringLit(value)) = self.peek().cloned() {
                                    attrs.insert(key, value);
                                    self.advance();
                                } else {
                                    // No value, store empty string
                                    attrs.insert(key, String::new());
                                }
                            } else {
                                // Key without value
                                attrs.insert(key, String::new());
                            }
                        } else {
                            // Not a string attribute, skip it
                            self.advance();
                        }
                    }
                    self.match_token(&Token::RBrace);
                }
                if !group_id.is_empty() {
                    self.attribute_groups.insert(group_id, attrs);
                }
                continue;
            }

            // Parse metadata definitions: !name = !{ ... } or !0 = !{...}
            let metadata_def_name = if let Some(Token::MetadataIdent(name)) = self.peek().cloned() {
                // Named metadata: !llvm.module.flags = ...
                let n = name.clone();
                self.advance();
                Some(n)
            } else if self.check(&Token::Exclaim) {
                // Might be numbered metadata: !0 = ...
                self.advance(); // consume !
                if let Some(Token::Integer(num)) = self.peek() {
                    let n = num.to_string();
                    self.advance(); // consume number
                    Some(n)
                } else {
                    // Not a metadata definition, put ! back
                    self.current -= 1;
                    None
                }
            } else {
                None
            };

            if let Some(metadata_name) = metadata_def_name {
                if self.match_token(&Token::Equal) {
                    // Parse the metadata content
                    if let Ok(metadata) = self.parse_metadata_node() {
                        // Store in registry for later reference
                        self.metadata_registry.insert(metadata_name.clone(), metadata.clone());

                        // Add to module's all_metadata collection
                        module.add_metadata(metadata_name.clone(), metadata.clone());

                        // Add to module if it's a named metadata (starts with "llvm.")
                        if metadata_name.starts_with("llvm.") {
                            // Named metadata can be a tuple of metadata nodes
                            if let Some(operands) = metadata.operands() {
                                module.add_named_metadata(metadata_name.clone(), operands.clone());
                            } else {
                                // Single metadata node - wrap in vector
                                module.add_named_metadata(metadata_name.clone(), vec![metadata.clone()]);
                            }
                        }
                    } else {
                        // If parsing fails, skip it
                        self.skip_metadata();
                    }
                    continue;
                }
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

        // Second pass: resolve metadata references and apply attribute groups
        self.resolve_metadata_references(&module);
        self.apply_attribute_groups(&module);

        Ok(module)
    }

    /// Apply attribute groups to functions
    fn apply_attribute_groups(&self, module: &Module) {
        for func in module.functions() {
            let attrs = func.attributes();
            for group_ref in &attrs.attribute_groups {
                if let Some(group_attrs) = self.attribute_groups.get(group_ref) {
                    for (key, value) in group_attrs {
                        func.add_string_attribute(key.clone(), value.clone());
                    }
                }
            }
        }
    }

    /// Resolve metadata forward references and populate module structures
    fn resolve_metadata_references(&self, module: &Module) {
        // Resolve all named metadata (llvm.*)
        // Named metadata like !llvm.ident = !{!0, !1} contains references that need resolution

        // Get a list of all llvm.* named metadata
        let named_metadata_keys: Vec<String> = self.metadata_registry.keys()
            .filter(|k| k.starts_with("llvm."))
            .cloned()
            .collect();

        for key in named_metadata_keys {
            if let Some(metadata) = self.metadata_registry.get(&key) {
                if let Some(refs) = metadata.operands() {
                    // Resolve all references and collect them
                    let resolved_metadata: Vec<_> = refs.iter()
                        .map(|ref_node| self.resolve_metadata_node(ref_node))
                        .collect();
                    // Replace with resolved versions
                    module.add_named_metadata(key.clone(), resolved_metadata);
                }
            }
        }

        // Handle !llvm.module.flags specifically for the module flags structure
        if let Some(module_flags_md) = self.metadata_registry.get("llvm.module.flags") {
            if let Some(flag_list) = module_flags_md.operands() {
                // Each element in flag_list is a reference to an actual flag
                // Resolve each reference and add to module
                for flag_ref in flag_list.iter() {
                    let resolved = self.resolve_metadata_node(flag_ref);
                    module.add_module_flag(resolved);
                }
            }
        }
    }

    /// Recursively resolve a metadata node, replacing references with actual content
    fn resolve_metadata_node(&self, node: &crate::metadata::Metadata) -> crate::metadata::Metadata {
        use crate::metadata::Metadata;

        // Check if this is a Reference that needs resolution
        if let Some(ref_name) = self.get_metadata_ref_name(node) {
            // Look up the referenced metadata in registry
            if let Some(target) = self.metadata_registry.get(&ref_name) {
                // Recursively resolve the target in case it contains more references
                return self.resolve_metadata_node(target);
            } else {
                // Couldn't resolve - return as-is
                return node.clone();
            }
        }

        // If it's a tuple, resolve all operands recursively
        if let Some(operands) = node.operands() {
            let resolved_operands: Vec<Metadata> = operands
                .iter()
                .map(|op| self.resolve_metadata_node(op))
                .collect();
            return Metadata::tuple(resolved_operands);
        }

        // For other types (String, Int, etc.), return as-is
        node.clone()
    }

    /// Helper to check if a metadata node is a Reference and get its name
    fn get_metadata_ref_name(&self, node: &crate::metadata::Metadata) -> Option<String> {
        node.as_reference().map(|s| s.to_string())
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

    fn parse_all_type_declarations(&mut self) -> ParseResult<()> {
        // Multi-pass approach to handle forward references
        // Keep parsing until no more type declarations are found or max iterations reached
        const MAX_PASSES: usize = 10;

        for _pass in 0..MAX_PASSES {
            self.current = 0;
            let mut found_any = false;

            while !self.is_at_end() {
                // Look for type declarations
                if (self.peek_global_ident().is_some() || self.check_local_ident())
                    && self.peek_ahead(1) == Some(&Token::Equal)
                    && self.peek_ahead(2) == Some(&Token::Type) {

                    // Try to parse this type declaration
                    let saved_pos = self.current;
                    if self.parse_type_declaration().is_ok() {
                        found_any = true;
                    } else {
                        // Parsing failed (probably forward reference), restore position
                        self.current = saved_pos;
                        self.advance();
                    }
                } else {
                    self.advance();
                }
            }

            // If no declarations were successfully parsed this pass, we're done
            if !found_any {
                break;
            }
        }

        Ok(())
    }

    fn parse_type_declaration(&mut self) -> ParseResult<()> {
        // %TypeName = type { ... } or %0 = type { ... } or %TypeName = type opaque
        let type_name = match self.peek() {
            Some(Token::GlobalIdent(name)) => name.clone(),
            Some(Token::LocalIdent(name)) => name.clone(),
            _ => return Err(ParseError::InvalidSyntax {
                message: "Expected type name".to_string(),
                position: self.current,
            }),
        };
        self.advance(); // consume type name
        self.consume(&Token::Equal)?;
        self.consume(&Token::Type)?;

        // Check if it's an opaque type
        let ty = if self.match_token(&Token::Opaque) {
            // Opaque type - create proper opaque type
            Type::opaque(&self.context, type_name.clone())
        } else {
            self.parse_type()?
        };

        // Store in type table
        self.type_table.insert(type_name, ty);
        Ok(())
    }

    fn parse_global_variable(&mut self) -> ParseResult<GlobalVariable> {
        use crate::module::{Linkage, Visibility, DLLStorageClass, ThreadLocalMode, UnnamedAddr};

        // @name = [linkage] [visibility] [dll_storage] [thread_local] [unnamed_addr] [addrspace] [externally_initialized] global/constant type [initializer] [attributes]
        let name = self.expect_global_ident()?;
        self.consume(&Token::Equal)?;

        // Parse linkage, visibility, and other attributes
        let mut linkage = Linkage::External;
        let mut visibility = Visibility::Default;
        let mut dll_storage_class = DLLStorageClass::Default;
        let mut thread_local_mode = ThreadLocalMode::NotThreadLocal;
        let mut unnamed_addr = UnnamedAddr::None;
        let mut externally_initialized = false;
        let mut addrspace = None;
        let mut section = None;
        let mut alignment = None;
        let mut comdat = None;

        // Parse attributes in a loop since they can appear in any order
        loop {
            match self.peek() {
                Some(Token::Private) => { self.advance(); linkage = Linkage::Private; },
                Some(Token::Internal) => { self.advance(); linkage = Linkage::Internal; },
                Some(Token::External) => { self.advance(); linkage = Linkage::External; },
                Some(Token::Weak) => { self.advance(); linkage = Linkage::Weak; },
                Some(Token::Linkonce) => { self.advance(); linkage = Linkage::Linkonce; },
                Some(Token::Linkonce_odr) => { self.advance(); linkage = Linkage::LinkonceOdr; },
                Some(Token::Weak_odr) => { self.advance(); linkage = Linkage::WeakOdr; },
                Some(Token::Available_externally) => { self.advance(); linkage = Linkage::AvailableExternally; },
                Some(Token::Extern_weak) => { self.advance(); linkage = Linkage::ExternWeak; },
                Some(Token::Common) => { self.advance(); linkage = Linkage::Common; },
                Some(Token::Appending) => { self.advance(); linkage = Linkage::Appending; },

                Some(Token::Hidden) => { self.advance(); visibility = Visibility::Hidden; },
                Some(Token::Protected) => { self.advance(); visibility = Visibility::Protected; },
                Some(Token::Default) => { self.advance(); visibility = Visibility::Default; },

                Some(Token::Dllimport) => { self.advance(); dll_storage_class = DLLStorageClass::DllImport; },
                Some(Token::Dllexport) => { self.advance(); dll_storage_class = DLLStorageClass::DllExport; },

                Some(Token::Thread_local) => {
                    self.advance();
                    // Check for thread local mode: thread_local(localdynamic)
                    if self.match_token(&Token::LParen) {
                        if let Some(Token::Identifier(mode)) = self.peek() {
                            match mode.as_str() {
                                "generaldynamic" => thread_local_mode = ThreadLocalMode::GeneralDynamic,
                                "localdynamic" => thread_local_mode = ThreadLocalMode::LocalDynamic,
                                "initialexec" => thread_local_mode = ThreadLocalMode::InitialExec,
                                "localexec" => thread_local_mode = ThreadLocalMode::LocalExec,
                                _ => thread_local_mode = ThreadLocalMode::GeneralDynamic,
                            }
                            self.advance();
                        }
                        self.match_token(&Token::RParen);
                    } else {
                        thread_local_mode = ThreadLocalMode::GeneralDynamic;
                    }
                },

                Some(Token::Unnamed_addr) => { self.advance(); unnamed_addr = UnnamedAddr::Global; },
                Some(Token::Local_unnamed_addr) => { self.advance(); unnamed_addr = UnnamedAddr::Local; },

                Some(Token::Dso_local) | Some(Token::Dso_preemptable) => { self.advance(); }, // Consume but don't store for now

                Some(Token::Addrspace) => {
                    self.advance();
                    if self.match_token(&Token::LParen) {
                        if let Some(Token::Integer(n)) = self.peek() {
                            // Address space must fit in 24 bits: max value is 16,777,215 (2^24 - 1)
                            if *n < 0 || *n >= (1 << 24) {
                                return Err(ParseError::InvalidSyntax {
                                    message: "invalid address space, must be a 24-bit integer".to_string(),
                                    position: self.current,
                                });
                            }
                            addrspace = Some(*n as u32);
                            self.advance();
                        } else if let Some(Token::StringLit(s)) = self.peek() {
                            // Symbolic address space: map to number
                            // Default mapping: A=1 (alloca), G=2 (global), P=3 (program)
                            let addr_num = match s.as_str() {
                                "A" => 1,
                                "G" => 2,
                                "P" => 3,
                                _ => 0, // Unknown symbolic addrspace defaults to 0
                            };
                            addrspace = Some(addr_num);
                            self.advance();
                        } else if !self.check(&Token::RParen) {
                            // Invalid token (like addrspace(D) or addrspace(@A))
                            // Skip it gracefully - defaults to addrspace 0
                            self.advance();
                        }
                        self.match_token(&Token::RParen);
                    }
                },

                Some(Token::Identifier(id)) if id == "externally_initialized" => {
                    self.advance();
                    externally_initialized = true;
                },

                // GPU calling conventions and other identifiers we should skip
                Some(Token::Identifier(id)) if
                    id.starts_with("amdgpu_") || id.starts_with("spir_") ||
                    id.starts_with("aarch64_") || id.starts_with("x86_") ||
                    id.starts_with("riscv_") || id.starts_with("arm_") ||
                    id == "linker_private" || id == "linker_private_weak" => {
                    self.advance();
                },

                _ => break,
            }
        }

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
        // Don't parse initializer if next token is:
        // - End of file
        // - A global ident (start of next global)
        // - Define/Declare (start of function)
        // - Comma (trailing attributes)
        // - A local ident that starts a type declaration (%T = type ...)
        let is_type_decl = self.check_local_ident()
            && self.peek_ahead(1) == Some(&Token::Equal)
            && self.peek_ahead(2) == Some(&Token::Type);

        let initializer = if !self.is_at_end() && !self.check_global_ident() && !self.check(&Token::Define) && !self.check(&Token::Declare) && !self.check(&Token::Comma) && !is_type_decl {
            // Parse the initializer and propagate errors
            Some(self.parse_global_initializer(&ty)?)
        } else {
            None
        };

        // Parse trailing attributes (section, align, comdat, etc.)
        loop {
            match self.peek() {
                Some(Token::Comma) => { self.advance(); },
                Some(Token::Section) => {
                    self.advance();
                    if let Some(Token::StringLit(s)) = self.peek() {
                        section = Some(s.clone());
                        self.advance();
                    }
                },
                Some(Token::Align) => {
                    self.advance();
                    if let Some(Token::Integer(n)) = self.peek() {
                        alignment = Some(*n as u32);
                        self.advance();
                    }
                },
                Some(Token::Comdat) => {
                    self.advance();
                    // Comdat can have an optional name: comdat($name) or comdat(identifier)
                    if self.match_token(&Token::LParen) {
                        // Handle $name format (GlobalIdent)
                        if let Some(Token::GlobalIdent(name)) = self.peek() {
                            comdat = Some(name.clone());
                            self.advance();
                        } else if let Some(Token::Identifier(name)) = self.peek() {
                            // Handle plain identifier format
                            comdat = Some(name.clone());
                            self.advance();
                        }
                        self.match_token(&Token::RParen);
                    } else {
                        // comdat without explicit name means use the global's own name
                        comdat = Some(name.clone());
                    }
                },
                _ => break,
            }
        }

        Ok(crate::module::GlobalVariable::new_with_attributes(
            name,
            ty,
            is_constant,
            initializer,
            linkage,
            visibility,
            dll_storage_class,
            thread_local_mode,
            unnamed_addr,
            addrspace,
            externally_initialized,
            section,
            alignment,
            comdat,
        ))
    }

    fn parse_alias(&mut self) -> ParseResult<crate::module::Alias> {
        use crate::module::{Linkage, Visibility, DLLStorageClass, ThreadLocalMode, UnnamedAddr, Alias};

        // @name = [linkage] [visibility] [dll_storage] [thread_local] [unnamed_addr] alias type, aliasee
        let name = self.expect_global_ident()?;
        self.consume(&Token::Equal)?;

        // Parse linkage, visibility, and other attributes
        let mut linkage = Linkage::External;
        let mut visibility = Visibility::Default;
        let mut dll_storage_class = DLLStorageClass::Default;
        let mut thread_local_mode = ThreadLocalMode::NotThreadLocal;
        let mut unnamed_addr = UnnamedAddr::None;

        // Parse attributes in a loop since they can appear in any order
        loop {
            match self.peek() {
                Some(Token::Private) => { self.advance(); linkage = Linkage::Private; },
                Some(Token::Internal) => { self.advance(); linkage = Linkage::Internal; },
                Some(Token::External) => { self.advance(); linkage = Linkage::External; },
                Some(Token::Weak) => { self.advance(); linkage = Linkage::Weak; },
                Some(Token::Linkonce) => { self.advance(); linkage = Linkage::Linkonce; },
                Some(Token::Linkonce_odr) => { self.advance(); linkage = Linkage::LinkonceOdr; },
                Some(Token::Weak_odr) => { self.advance(); linkage = Linkage::WeakOdr; },
                Some(Token::Available_externally) => { self.advance(); linkage = Linkage::AvailableExternally; },
                Some(Token::Extern_weak) => { self.advance(); linkage = Linkage::ExternWeak; },
                Some(Token::Common) => { self.advance(); linkage = Linkage::Common; },
                Some(Token::Appending) => { self.advance(); linkage = Linkage::Appending; },

                Some(Token::Hidden) => { self.advance(); visibility = Visibility::Hidden; },
                Some(Token::Protected) => { self.advance(); visibility = Visibility::Protected; },
                Some(Token::Default) => { self.advance(); visibility = Visibility::Default; },

                Some(Token::Dllimport) => { self.advance(); dll_storage_class = DLLStorageClass::DllImport; },
                Some(Token::Dllexport) => { self.advance(); dll_storage_class = DLLStorageClass::DllExport; },

                Some(Token::Thread_local) => {
                    self.advance();
                    if self.match_token(&Token::LParen) {
                        if let Some(Token::Identifier(mode)) = self.peek() {
                            match mode.as_str() {
                                "generaldynamic" => thread_local_mode = ThreadLocalMode::GeneralDynamic,
                                "localdynamic" => thread_local_mode = ThreadLocalMode::LocalDynamic,
                                "initialexec" => thread_local_mode = ThreadLocalMode::InitialExec,
                                "localexec" => thread_local_mode = ThreadLocalMode::LocalExec,
                                _ => thread_local_mode = ThreadLocalMode::GeneralDynamic,
                            }
                            self.advance();
                        }
                        self.match_token(&Token::RParen);
                    } else {
                        thread_local_mode = ThreadLocalMode::GeneralDynamic;
                    }
                },

                Some(Token::Unnamed_addr) => { self.advance(); unnamed_addr = UnnamedAddr::Global; },
                Some(Token::Local_unnamed_addr) => { self.advance(); unnamed_addr = UnnamedAddr::Local; },

                Some(Token::Dso_local) | Some(Token::Dso_preemptable) => { self.advance(); },

                _ => break,
            }
        }

        // Expect 'alias' keyword
        self.consume(&Token::Alias)?;

        // Parse alias type
        let alias_type = self.parse_type()?;

        // Expect comma
        self.consume(&Token::Comma)?;

        // Parse aliasee - could be: type value OR constant_expression
        // Check if next token is a constant expression (no type prefix)
        let aliasee = match self.peek() {
            Some(Token::GetElementPtr) | Some(Token::BitCast) | Some(Token::AddrSpaceCast) |
            Some(Token::PtrToInt) | Some(Token::IntToPtr) => {
                // Constant expression without type prefix
                self.parse_value()?
            }
            _ => {
                // Normal case: type value
                let aliasee_type = self.parse_type()?;
                self.parse_value_with_type(Some(&aliasee_type))?
            }
        };

        Ok(Alias {
            name,
            ty: alias_type,
            aliasee,
            linkage,
            visibility,
            dll_storage_class,
            thread_local_mode,
            unnamed_addr,
        })
    }

    fn parse_global_initializer(&mut self, ty: &Type) -> ParseResult<Value> {
        // Parse common initializer forms
        match self.peek() {
            Some(Token::Zeroinitializer) => {
                self.advance();
                Ok(Value::zero_initializer(ty.clone()))
            },
            Some(Token::Undef) => {
                self.advance();
                Ok(Value::undef(ty.clone()))
            },
            Some(Token::Null) => {
                self.advance();
                Ok(Value::const_null(ty.clone()))
            },
            Some(Token::Integer(_)) => {
                if let Some(Token::Integer(n)) = self.peek() {
                    let val = *n;
                    self.advance();
                    if ty.is_integer() {
                        Ok(Value::const_int(ty.clone(), val as i64, None))
                    } else if ty.is_float() {
                        // Integer literal with float type - interpret bits as float
                        // e.g., double 0x8000000000000000 = -0.0
                        let float_val = if ty == &self.context.double_type() {
                            f64::from_bits(val as u64)
                        } else {
                            // float type
                            f32::from_bits(val as u32) as f64
                        };
                        Ok(Value::const_float(ty.clone(), float_val, None))
                    } else {
                        // Other types - try as constant expression
                        self.parse_constant_expression()
                    }
                } else {
                    unreachable!()
                }
            },
            Some(Token::True) => {
                self.advance();
                if ty.is_integer() {
                    Ok(Value::const_int(ty.clone(), 1, None))
                } else {
                    Ok(Value::const_int(self.context.bool_type(), 1, None))
                }
            },
            Some(Token::False) => {
                self.advance();
                if ty.is_integer() {
                    Ok(Value::const_int(ty.clone(), 0, None))
                } else {
                    Ok(Value::const_int(self.context.bool_type(), 0, None))
                }
            },
            Some(Token::Float64(_)) => {
                if let Some(Token::Float64(f)) = self.peek() {
                    let val = *f;
                    self.advance();
                    Ok(Value::const_float(ty.clone(), val, None))
                } else {
                    unreachable!()
                }
            },
            Some(Token::LBrace) | Some(Token::LBracket) | Some(Token::StringLit(_)) | Some(Token::CString(_)) => {
                // Complex aggregate constant - parse with expected type for validation
                self.parse_value_with_type(Some(ty))
            },
            Some(Token::Identifier(_)) => {
                // Could be splat, asm, or other special identifiers
                self.parse_value_with_type(Some(ty))
            },
            Some(Token::Ptrauth) => {
                // ptrauth (ptr value, i32 key [, i64 discriminator [, ptr address_discriminator]])
                self.parse_ptrauth_constant()
            },
            _ => {
                // Try parsing as a constant expression
                self.parse_constant_expression()
            }
        }
    }

    fn parse_ptrauth_constant(&mut self) -> ParseResult<Value> {
        // ptrauth (ptr value, i32 key [, i64 discriminator [, ptr address_discriminator]])
        self.consume(&Token::Ptrauth)?;
        self.consume(&Token::LParen)?;

        // Parse pointer value with its type
        let ptr_ty = self.parse_type()?;
        let ptr_value = self.parse_value_with_type(Some(&ptr_ty))?;

        self.consume(&Token::Comma)?;

        // Parse key (i32)
        let key_ty = self.parse_type()?;
        let _key_value = self.parse_value_with_type(Some(&key_ty))?;

        // Parse optional discriminator (i64)
        if self.match_token(&Token::Comma) {
            let disc_ty = self.parse_type()?;
            let _disc_value = self.parse_value_with_type(Some(&disc_ty))?;

            // Parse optional address discriminator (ptr)
            if self.match_token(&Token::Comma) {
                let addr_ty = self.parse_type()?;
                let _addr_value = self.parse_value_with_type(Some(&addr_ty))?;
            }
        }

        self.consume(&Token::RParen)?;

        // For now, return the pointer value as-is (ptrauth is essentially a signed pointer)
        // In a full implementation, we would preserve the ptrauth metadata
        Ok(ptr_value)
    }

    fn parse_function_declaration(&mut self) -> ParseResult<Function> {
        // declare [visibility] [dll_storage] [cc] [ret attrs] [!metadata] type @name([params])
        let visibility = self.parse_visibility();
        let dll_storage_class = self.parse_dll_storage_class();
        let cc = self.parse_calling_convention();
        let ret_attrs = self.parse_return_attributes();

        // Skip metadata attachments before return type (e.g., declare !dbg !12 i32 @foo())
        while self.is_metadata_token() {
            self.skip_metadata();
        }

        let return_type = self.parse_type()?;
        let name = self.expect_global_ident()?;

        self.consume(&Token::LParen)?;
        let (param_types, param_attrs, is_vararg) = self.parse_parameter_types()?;
        self.consume(&Token::RParen)?;

        // Parse function attributes
        let mut attrs = self.parse_function_attributes();
        attrs.return_attributes = ret_attrs;
        attrs.parameter_attributes = param_attrs;

        let fn_type = self.context.function_type(return_type, param_types, is_vararg);
        let function = Function::new(name, fn_type);
        function.set_visibility(visibility);
        function.set_dll_storage_class(dll_storage_class);
        function.set_calling_convention(cc);
        function.set_attributes(attrs);
        Ok(function)
    }

    fn parse_function_definition(&mut self) -> ParseResult<Function> {
        // define [linkage] [visibility] [dll_storage] [cc] [ret attrs] [!metadata] type @name([params]) [fn attrs] { body }
        let linkage = self.parse_linkage();
        let visibility = self.parse_visibility();
        let dll_storage_class = self.parse_dll_storage_class();
        let cc = self.parse_calling_convention();
        let ret_attrs = self.parse_return_attributes();

        // Skip metadata attachments before return type
        while self.is_metadata_token() {
            self.skip_metadata();
        }

        let return_type = self.parse_type()?;
        let name = self.expect_global_ident()?;

        self.consume(&Token::LParen)?;
        let (params, param_attrs, is_vararg) = self.parse_parameters()?;
        self.consume(&Token::RParen)?;

        // Parse function attributes
        let mut attrs = self.parse_function_attributes();
        attrs.return_attributes = ret_attrs;
        attrs.parameter_attributes = param_attrs;

        // Create function
        let param_types: Vec<Type> = params.iter().map(|(ty, _)| ty.clone()).collect();
        let fn_type = self.context.function_type(return_type, param_types, is_vararg);
        let function = Function::new(name, fn_type);
        function.set_linkage(linkage);
        function.set_visibility(visibility);
        function.set_dll_storage_class(dll_storage_class);
        function.set_calling_convention(cc);
        function.set_attributes(attrs);

        // Set arguments
        let args: Vec<Value> = params.iter().enumerate().map(|(idx, (ty, name))| {
            Value::argument(ty.clone(), idx, Some(name.clone()))
        }).collect();
        function.set_arguments(args.clone());

        // Clear symbol table for this function and populate with parameters
        self.symbol_table.clear();
        for arg in &args {
            if let Some(name) = arg.name() {
                self.symbol_table.insert(name.to_string(), arg.clone());
            }
        }

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
                    Token::StringLit(s) => Some(s), // String labels like "2":
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
            }
            // Check for multi-token labels like -3:, -N-:, $N:
            // Look ahead a few tokens to see if there's a colon
            else {
                let mut colon_pos = None;
                for i in 1..=5 {
                    if self.peek_ahead(i) == Some(&Token::Colon) {
                        colon_pos = Some(i);
                        break;
                    }
                }

                if let Some(pos) = colon_pos {
                    // Found a colon within 5 tokens, treat this as a multi-token label
                    let mut label_name = String::new();
                    for _ in 0..pos {
                        if let Some(tok) = self.peek() {
                            // Build label name from tokens
                            match tok {
                                Token::Sub => label_name.push('-'),
                                Token::Integer(n) => label_name.push_str(&n.to_string()),
                                Token::Identifier(s) => label_name.push_str(s),
                                Token::StringLit(s) => label_name.push_str(s),
                                _ => label_name.push_str(&format!("{:?}", tok)),
                            }
                        }
                        self.advance();
                    }
                    self.advance(); // consume colon
                    Some(label_name)
                } else {
                    // Entry block without label
                    None
                }
            }
        } else {
            None
        };

        // If we didn't find a label and we're at end of function, return None
        if name.is_none() && (self.check(&Token::RBrace) || self.is_at_end()) {
            return Ok(None);
        }

        let bb = BasicBlock::new(name.clone());

        // Parse instructions with iteration limit to prevent infinite loops
        let mut inst_count = 0;
        let mut last_position = self.current;
        let mut stuck_count = 0;
        const MAX_INSTRUCTIONS_PER_BLOCK: usize = 50000;  // Increased for large basic blocks
        const MAX_STUCK_ITERATIONS: usize = 10;  // Increased to handle complex edge cases

        loop {
            if inst_count >= MAX_INSTRUCTIONS_PER_BLOCK {
                return Err(ParseError::InvalidSyntax {
                    message: format!("Basic block exceeded maximum instruction count ({}), possible infinite loop", MAX_INSTRUCTIONS_PER_BLOCK),
                    position: self.current,
                });
            }

            // Detect if we're stuck on the same token
            if self.current == last_position {
                stuck_count += 1;
                if stuck_count >= MAX_STUCK_ITERATIONS {
                    let stuck_token = self.peek().map(|t| format!("{:?}", t)).unwrap_or_else(|| "EOF".to_string());
                    return Err(ParseError::InvalidSyntax {
                        message: format!("Parser stuck at token position {} on token: {}", self.current, stuck_token),
                        position: self.current,
                    });
                }
            } else {
                stuck_count = 0;
                last_position = self.current;
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
                // parse_instruction returned None - this can mean:
                // 1. We skipped a directive (debug record, uselistorder, etc.) - continue parsing
                // 2. We're at the end of the block - the checks above will catch this next iteration
                // 3. First iteration of unlabeled block with nothing - return None to avoid infinite loop
                if inst_count == 1 && name.is_none() && bb.instructions().is_empty() {
                    // Check if we're truly at end or just skipped something
                    if self.check(&Token::RBrace) || self.is_at_end() ||
                       self.peek_ahead(1) == Some(&Token::Colon) {
                        return Ok(None);
                    }
                }
                // Otherwise continue - we probably just skipped a directive
            }
        }

        // Don't return empty unnamed blocks (can happen after uselistorder directives)
        if name.is_none() && bb.instructions().is_empty() {
            return Ok(None);
        }

        Ok(Some(bb))
    }

    fn parse_instruction(&mut self) -> ParseResult<Option<Instruction>> {
        // Skip uselistorder directives: uselistorder type value, { order... }
        if let Some(Token::Identifier(id)) = self.peek() {
            if id == "uselistorder" || id == "uselistorder_bb" {
                self.advance(); // skip uselistorder
                // Skip the rest of the directive until we hit a closing brace
                let mut brace_depth = 0;
                while !self.is_at_end() {
                    if self.check(&Token::LBrace) {
                        brace_depth += 1;
                        self.advance();
                    } else if self.check(&Token::RBrace) {
                        if brace_depth == 0 {
                            break; // End of function
                        }
                        brace_depth -= 1;
                        self.advance();
                        if brace_depth == 0 {
                            break; // End of uselistorder
                        }
                    } else {
                        self.advance();
                    }
                }
                return Ok(None);
            }
        }

        // Check for result assignment: %name = ... (must be done BEFORE skipping modifiers)

        let result_name = if let Some(Token::LocalIdent(n)) = self.peek().cloned() {
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

        // Skip calling convention modifiers (tail, musttail, notail) - these modify the following call
        if let Some(Token::Identifier(id)) = self.peek() {
            if id == "tail" || id == "musttail" || id == "notail" {
                self.advance(); // skip the modifier, but continue parsing the call
            }
        }

        // Skip standalone function attribute keywords that might appear in basic blocks
        if self.check(&Token::Nobuiltin) || self.check(&Token::Builtin) ||
           self.check(&Token::Cold) || self.check(&Token::Hot) ||
           self.check(&Token::Noduplicate) || self.check(&Token::Noimplicitfloat) ||
           self.check(&Token::Noinline) || self.check(&Token::Strictfp) ||
           self.check(&Token::Minsize) || self.check(&Token::Alwaysinline) ||
           self.check(&Token::Optsize) || self.check(&Token::Optnone) {
            self.advance();
            return Ok(None);
        }

        // Check for debug records: #dbg_declare, #dbg_value, etc. or dbg_declare (if # was consumed elsewhere)
        if self.check(&Token::Hash) {
            // Check if this is followed by an identifier (debug intrinsic)
            if matches!(self.peek_ahead(1), Some(Token::Identifier(_))) {
                self.advance(); // consume #
                self.advance(); // skip identifier (dbg_declare/dbg_value/etc)
                // Skip the argument list if present
                if self.check(&Token::LParen) {
                    self.advance(); // consume (
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
                return Ok(None);
            }
        } else if let Some(Token::Identifier(id)) = self.peek() {
            // Handle debug intrinsics where # was already consumed
            if id.starts_with("dbg_") {
                self.advance(); // skip identifier
                // Skip the argument list if present
                if self.check(&Token::LParen) {
                    self.advance(); // consume (
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
                return Ok(None);
            }
        }

        // Parse instruction opcode
        let opcode = if let Some(op) = self.parse_opcode()? {
            op
        } else {
            return Ok(None);
        };

        // Parse operands and get result type if instruction produces one
        let (operands, result_type, gep_source_type, alignment, is_atomic) = self.parse_instruction_operands(opcode)?;

        // Skip instruction-level attributes that come after operands (nounwind, readonly, etc.)
        self.skip_instruction_level_attributes();

        // Parse metadata attachments after instructions and track their names
        // Some instructions (like extractvalue) may consume a comma but leave metadata
        // Handle both: ", !foo !0" and "!foo !0" (comma already consumed)
        let mut metadata_attachments = Vec::new();
        loop {
            if self.match_token(&Token::Comma) {
                // Comma-prefixed metadata: , !dbg !0
                if self.is_metadata_token() {
                    if let Some(name) = self.skip_metadata() {
                        metadata_attachments.push(name);
                    }
                    if self.is_metadata_token() {
                        self.skip_metadata(); // Skip the metadata value (!0, !{}, etc.)
                    }
                } else {
                    // Not metadata, put comma back and stop
                    self.current -= 1;
                    break;
                }
            } else if self.is_metadata_token() {
                // Direct metadata (comma was consumed by operand parsing)
                if let Some(name) = self.skip_metadata() {
                    metadata_attachments.push(name);
                }
                if self.is_metadata_token() {
                    self.skip_metadata(); // Skip the metadata value (!0, !{}, etc.)
                }
            } else {
                // No more metadata
                break;
            }
        }

        // Create result value if there's a result name OR if instruction produces a non-void result
        let result = if let Some(name) = result_name {
            // Named result
            let ty = result_type.unwrap_or_else(|| self.context.void_type());
            let value = Value::instruction(ty, opcode, Some(name.clone()));
            // Add to symbol table for lookup
            self.symbol_table.insert(name, value.clone());
            Some(value)
        } else if let Some(ty) = result_type {
            // Unnamed result but non-void type (e.g., call that returns value but result unused)
            if !ty.is_void() {
                Some(Value::instruction(ty, opcode, None))
            } else {
                None
            }
        } else {
            None
        };

        let mut inst = Instruction::new(opcode, operands, result);

        // Set GEP source type if this is a GetElementPtr instruction
        if let Some(gep_type) = gep_source_type {
            inst.set_gep_source_type(gep_type);
        }

        // Set alignment if specified
        if let Some(align) = alignment {
            inst.set_alignment(align);
        }

        // Set atomic flag if specified
        if is_atomic {
            inst.set_atomic(true);
        }

        // Attach metadata to the instruction
        for md_name in metadata_attachments {
            inst.add_metadata_attachment(md_name);
        }
        Ok(Some(inst))
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
            Token::Freeze => { self.advance(); Opcode::Freeze }
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
            Token::PtrToAddr => { self.advance(); Opcode::PtrToAddr }
            Token::AddrToPtr => { self.advance(); Opcode::AddrToPtr }
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

    fn parse_instruction_operands(&mut self, opcode: Opcode) -> ParseResult<(Vec<Value>, Option<Type>, Option<Type>, Option<u64>, bool)> {
        let mut operands = Vec::new();
        let mut result_type: Option<Type> = None;
        let mut gep_source_type_field: Option<Type> = None;
        let mut alignment: Option<u64> = None;
        let mut is_atomic = false;

        // Parse based on instruction type
        match opcode {
            Opcode::Ret => {
                // ret void or ret type value
                // Parse type first (handles both plain "void" and complex types like "void ()*")
                if !self.is_at_end() && !self.check(&Token::RBrace) &&
                   self.peek_ahead(1) != Some(&Token::Colon) {
                    let ty = self.parse_type()?;
                    // After parsing type, try to parse a value if present
                    // Don't parse if we're at end of block or the next token looks like a label
                    if !self.is_at_end() && !self.check(&Token::RBrace) {
                        // Check if current token is followed by colon (label definition)
                        if self.peek_ahead(1) == Some(&Token::Colon) {
                            // This is a label, not a return value
                            return Ok((operands, result_type, None, None, false));
                        }
                        // Try to parse a value - if the type is void, there might not be one
                        if !ty.is_void() {
                            let val = self.parse_value_with_type(Some(&ty))?;
                            operands.push(val);
                        }
                    }
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
                // call [fast-math-flags] [cc] [attrs] type [(param_types...)] @func(args...)
                // Parse in the correct order:
                // 1. Fast-math flags (nnan, ninf, etc.)
                self.skip_instruction_flags();
                // 2. Calling convention (fastcc, coldcc, etc.)
                self.skip_linkage_and_visibility();
                // 3. Return attributes (inreg, zeroext, etc.)
                self.skip_attributes();

                let ret_ty = self.parse_type()?;

                // If parse_type returned a function type (e.g., i32 (i8*, ...)), extract components
                let (return_type, explicit_fn_type) = if ret_ty.is_function() {
                    // parse_type already parsed the full function signature
                    if let Some((ret, _, _)) = ret_ty.function_info() {
                        (ret, Some(ret_ty.clone()))
                    } else {
                        (ret_ty.clone(), None)
                    }
                } else {
                    // Just a return type, no explicit function signature
                    (ret_ty.clone(), None)
                };

                result_type = Some(return_type);  // Call result type is the return type

                // Parse function value (which may include dso_local_equivalent, no_cfi, etc.)
                let func = if let Some(ref fn_ty) = explicit_fn_type {
                    self.parse_value_with_type(Some(fn_ty))?
                } else {
                    self.parse_value()?
                };
                operands.push(func);
                self.consume(&Token::LParen)?;
                // Determine if this is a varargs call
                let is_varargs = if let Some(fn_ty) = &explicit_fn_type {
                    fn_ty.function_info().map(|(_, _, varargs)| varargs).unwrap_or(false)
                } else {
                    false
                };
                let args = self.parse_call_arguments_with_context(is_varargs)?;
                // Extract values from (Type, Value) pairs
                for (_, value) in args {
                    operands.push(value);
                }
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
                        // Need to handle nested parentheses for function pointer types
                        if self.check(&Token::LParen) {
                            self.advance(); // consume opening (
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
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.match_token(&Token::RBracket);
                }

                // Skip function attributes that may appear after arguments (nounwind, readonly, etc.)
                self.skip_function_attributes();
            }
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::UDiv | Opcode::SDiv |
            Opcode::URem | Opcode::SRem | Opcode::Shl | Opcode::LShr | Opcode::AShr |
            Opcode::And | Opcode::Or | Opcode::Xor |
            Opcode::FAdd | Opcode::FSub | Opcode::FMul | Opcode::FDiv | Opcode::FRem => {
                // Binary ops: op [flags] type op1, op2
                self.skip_instruction_flags();
                let ty = self.parse_type()?;
                result_type = Some(ty.clone());  // Binary op result has same type as operands
                let op1 = self.parse_value_with_type(Some(&ty))?;
                operands.push(op1);
                self.consume(&Token::Comma)?;
                let op2 = self.parse_value_with_type(Some(&ty))?;
                operands.push(op2);
            }
            Opcode::Alloca => {
                // alloca [inalloca] [swifterror] type [, type NumElements] [, align N] [, addrspace(N)]
                // Skip optional inalloca and swifterror keywords
                self.match_token(&Token::Inalloca);
                self.match_token(&Token::Swifterror);

                let alloca_ty = self.parse_type()?;

                // Validate that alloca type is sized (not void, function, label, token, or metadata)
                // Note: target types are represented as opaque but ARE allocatable
                let is_target_type = format!("{:?}", alloca_ty).starts_with("Type(%target(");
                if !alloca_ty.is_sized() && !is_target_type {
                    return Err(ParseError::InvalidSyntax {
                        message: format!("invalid type for alloca: {:?}", alloca_ty),
                        position: self.current,
                    });
                }

                // Alloca returns a pointer to the allocated type
                result_type = Some(self.context.ptr_type(alloca_ty));

                // Handle optional attributes in any order
                while self.match_token(&Token::Comma) {
                    if self.match_token(&Token::Align) {
                        if let Some(Token::Integer(val)) = self.peek() {
                            alignment = Some(*val as u64);
                            self.advance();
                        }
                    } else if self.match_token(&Token::Addrspace) {
                        self.consume(&Token::LParen)?;
                        // Address space can be integer or symbolic string ("A", "G", "P")
                        // Address space must fit in 24 bits: max value is 16,777,215 (2^24 - 1)
                        if let Some(Token::Integer(val)) = self.peek() {
                            if *val < 0 || *val >= (1 << 24) {
                                return Err(ParseError::InvalidSyntax {
                                    message: "invalid address space, must be a 24-bit integer".to_string(),
                                    position: self.current,
                                });
                            }
                            self.advance();
                        } else if let Some(Token::StringLit(_)) = self.peek() {
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
                // Old syntax: load atomic i32* %ptr (no comma, typed pointer)
                // New syntax: load atomic i32, ptr %ptr (comma-separated)
                is_atomic = self.match_token(&Token::Atomic);
                self.match_token(&Token::Volatile);

                let ty = self.parse_type()?;
                result_type = Some(ty);  // Load result type is the loaded type

                // Check if using old syntax (no comma) or new syntax (comma)
                if self.match_token(&Token::Comma) {
                    // New syntax: comma separates type from pointer
                    let _ptr_ty = self.parse_type()?;
                    let ptr = self.parse_value()?;
                    operands.push(ptr);
                } else {
                    // Old syntax: pointer value directly follows (type already includes *)
                    let ptr = self.parse_value()?;
                    operands.push(ptr);
                }

                // Parse memory ordering and alignment attributes
                alignment = self.parse_load_store_attributes();
            }
            Opcode::Store => {
                // store [atomic] [volatile] type %val, ptr %ptr [, align ...]
                // Old syntax: store i32 %val, i32* %ptr (typed pointer)
                // New syntax: store i32 %val, ptr %ptr (opaque pointer)
                is_atomic = self.match_token(&Token::Atomic);
                self.match_token(&Token::Volatile);

                let val_ty = self.parse_type()?;
                let val = self.parse_value_with_type(Some(&val_ty))?;
                operands.push(val);
                self.consume(&Token::Comma)?;

                // Parse pointer - could be "ptr" keyword (new) or typed pointer (old)
                let _ptr_ty = self.parse_type()?;
                let ptr = self.parse_value()?;
                operands.push(ptr);

                // Parse alignment and other attributes
                alignment = self.parse_load_store_attributes();
            }
            Opcode::GetElementPtr => {
                // getelementptr [inbounds] [nuw] [nusw] type, ptr %ptr, indices...
                self.skip_instruction_flags(); // Skip inbounds, nuw, nusw, etc.
                let gep_source_type = self.parse_type()?;
                gep_source_type_field = Some(gep_source_type.clone());
                self.consume(&Token::Comma)?;
                let ptr_ty = self.parse_type()?;
                let ptr = self.parse_value_with_type(Some(&ptr_ty))?;
                operands.push(ptr);

                // GEP result type determination:
                // - If base is <N x ptr>, result is <N x ptr>
                // - If any index is <N x iXX>, result is <N x ptr>
                // - Otherwise, result is ptr
                let mut vector_size = None;

                // Check if base is a vector
                if ptr_ty.is_vector() {
                    vector_size = ptr_ty.vector_info().map(|(_, size)| size);
                }

                // Parse indices and check if any are vectors
                while self.match_token(&Token::Comma) {
                    // Check if this comma is followed by metadata (e.g., !dbg !23)
                    if self.is_metadata_token() {
                        // Put comma back and let instruction-level metadata handler deal with it
                        self.current -= 1;
                        break;
                    }
                    self.match_token(&Token::Inrange); // Skip optional inrange
                    let idx_ty = self.parse_type()?;

                    // Check if this index is a vector
                    if idx_ty.is_vector() {
                        vector_size = idx_ty.vector_info().map(|(_, size)| size);
                    }

                    let idx = self.parse_value_with_type(Some(&idx_ty))?;
                    operands.push(idx);
                }

                // Set result type based on whether we found a vector
                result_type = if let Some(size) = vector_size {
                    Some(self.context.vector_type(
                        self.context.ptr_type(self.context.int8_type()),
                        size
                    ))
                } else {
                    Some(self.context.ptr_type(self.context.int8_type()))
                };
            }
            Opcode::ICmp | Opcode::FCmp => {
                // icmp/fcmp [samesign] predicate type op1, op2
                self.skip_instruction_flags(); // Skip flags like samesign
                self.parse_comparison_predicate()?;
                let ty = self.parse_type()?;
                // Comparison result is i1 for scalars, <N x i1> for vectors
                let cmp_result_ty = if ty.is_vector() {
                    if let Some((_, size)) = ty.vector_info() {
                        self.context.vector_type(self.context.bool_type(), size)
                    } else {
                        self.context.bool_type()
                    }
                } else {
                    self.context.bool_type()
                };
                result_type = Some(cmp_result_ty);
                let op1 = self.parse_value_with_type(Some(&ty))?;
                operands.push(op1);
                self.consume(&Token::Comma)?;
                let op2 = self.parse_value_with_type(Some(&ty))?;
                operands.push(op2);
            }
            Opcode::PHI => {
                // phi [fast-math-flags] type [ val1, %bb1 ], [ val2, %bb2 ], ...
                self.skip_instruction_flags();
                let ty = self.parse_type()?;
                result_type = Some(ty.clone());
                while !self.is_at_end() {
                    if !self.match_token(&Token::LBracket) {
                        break;
                    }
                    let val = self.parse_value_with_type(Some(&ty))?;
                    operands.push(val);
                    self.consume(&Token::Comma)?;
                    let bb = self.expect_local_ident()?;
                    let bb_label = Value::new(
                        self.context.label_type(),
                        crate::value::ValueKind::BasicBlock,
                        Some(bb)
                    );
                    operands.push(bb_label);
                    self.consume(&Token::RBracket)?;
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            Opcode::Trunc | Opcode::ZExt | Opcode::SExt | Opcode::FPTrunc | Opcode::FPExt |
            Opcode::FPToUI | Opcode::FPToSI | Opcode::UIToFP | Opcode::SIToFP |
            Opcode::PtrToInt | Opcode::IntToPtr | Opcode::PtrToAddr | Opcode::AddrToPtr |
            Opcode::BitCast | Opcode::AddrSpaceCast => {
                // cast [flags] type1 %val to type2
                self.skip_instruction_flags(); // Skip fast-math flags for FP casts
                let src_ty = self.parse_type()?;
                let val = self.parse_value_with_type(Some(&src_ty))?;
                operands.push(val);
                self.consume(&Token::To)?;
                let dest_ty = self.parse_type()?;
                result_type = Some(dest_ty);  // Cast result type is the destination type
            }
            Opcode::ExtractElement => {
                // extractelement <vector type> %vec, <index type> %idx
                let vec_ty = self.parse_type()?;
                let vec = self.parse_value_with_type(Some(&vec_ty))?;
                operands.push(vec);
                self.consume(&Token::Comma)?;
                let idx_ty = self.parse_type()?;
                let idx = self.parse_value_with_type(Some(&idx_ty))?;
                operands.push(idx);

                // Result type is the element type of the vector
                // For vector types like <2 x i8>, the element type is i8
                if let Some((elem_ty, _size)) = vec_ty.vector_info() {
                    result_type = Some(elem_ty.clone());
                }
            }
            Opcode::InsertElement => {
                // insertelement <vector type> %vec, <element type> %elt, <index type> %idx
                let vec_ty = self.parse_type()?;
                let vec = self.parse_value_with_type(Some(&vec_ty))?;
                operands.push(vec);
                self.consume(&Token::Comma)?;
                let elt_ty = self.parse_type()?;
                let elt = self.parse_value_with_type(Some(&elt_ty))?;
                operands.push(elt);
                self.consume(&Token::Comma)?;
                let idx_ty = self.parse_type()?;
                let idx = self.parse_value_with_type(Some(&idx_ty))?;
                operands.push(idx);

                // Result type is the same as the input vector type
                result_type = Some(vec_ty.clone());
            }
            Opcode::Select => {
                // select [fast-math-flags] i1 %cond, type %val1, type %val2
                self.skip_instruction_flags();
                let cond_ty = self.parse_type()?;
                let cond = self.parse_value_with_type(Some(&cond_ty))?;
                operands.push(cond);
                self.consume(&Token::Comma)?;
                let ty1 = self.parse_type()?;
                let val1 = self.parse_value_with_type(Some(&ty1))?;
                operands.push(val1);
                self.consume(&Token::Comma)?;
                let ty2 = self.parse_type()?;
                let val2 = self.parse_value_with_type(Some(&ty2))?;
                operands.push(val2);
                result_type = Some(ty1);
            }
            Opcode::AtomicCmpXchg => {
                // cmpxchg [weak] [volatile] ptr <pointer>, type <cmp>, type <new> [syncscope] <ordering> <ordering> [, align N] [, !metadata !0]
                self.match_token(&Token::Weak);
                self.match_token(&Token::Volatile);

                // Parse pointer type and value
                let _ptr_ty = self.parse_type()?;
                let _ptr = self.parse_value()?;
                self.consume(&Token::Comma)?;

                // Parse compare type and value
                let cmp_ty = self.parse_type()?;
                let _cmp = self.parse_value()?;
                self.consume(&Token::Comma)?;

                // Parse new type and value
                let _new_ty = self.parse_type()?;
                let _new = self.parse_value()?;

                // cmpxchg returns { type, i1 } - old value and success flag
                let struct_ty = crate::types::Type::struct_type(
                    &self.context,
                    vec![cmp_ty, self.context.bool_type()],
                    None
                );
                result_type = Some(struct_ty);

                // Skip syncscope if present
                self.skip_syncscope();

                // Parse two memory orderings (as keyword tokens, not identifiers)
                self.skip_memory_ordering();
                self.skip_memory_ordering();

                // Handle optional align parameter using same logic as load/store
                if self.match_token(&Token::Comma) {
                    if self.match_token(&Token::Align) {
                        if let Some(Token::Integer(_)) = self.peek() {
                            self.advance();
                        }
                    } else {
                        // Not align, put comma back so metadata handling can process it
                        self.current -= 1;
                    }
                }
            }
            Opcode::AtomicRMW => {
                // atomicrmw [volatile] <operation> ptr <pointer>, type <value> [syncscope] <ordering> [, align N] [, !metadata !0]
                self.match_token(&Token::Volatile);

                // Parse operation (xchg, add, sub, and, or, xor, max, min, umax, umin, etc.)
                // These can be opcodes (Add, Sub, etc.) or identifiers (xchg, max, min, etc.)
                if let Some(token) = self.peek() {
                    match token {
                        // Match known atomic RMW operation tokens/keywords
                        Token::Add | Token::Sub | Token::And | Token::Or | Token::Xor |
                        Token::Identifier(_) => {
                            self.advance(); // Skip the operation
                        }
                        _ => {
                            // Unknown operation, try to skip anyway
                            self.advance();
                        }
                    }
                } else {
                    return Err(ParseError::UnexpectedEOF);
                }

                // Parse pointer type and value
                let _ptr_ty = self.parse_type()?;
                let _ptr = self.parse_value()?;
                self.consume(&Token::Comma)?;

                // Parse value type and value
                let val_ty = self.parse_type()?;
                let _val = self.parse_value()?;

                // atomicrmw returns the old value, same type as the value parameter
                result_type = Some(val_ty);

                // Skip syncscope if present
                self.skip_syncscope();

                // Parse ordering (as keyword token, not identifier)
                self.skip_memory_ordering();

                // Handle optional align parameter using same logic as load/store
                if self.match_token(&Token::Comma) {
                    if self.match_token(&Token::Align) {
                        if let Some(Token::Integer(_)) = self.peek() {
                            self.advance();
                        }
                    } else {
                        // Not align, put comma back so metadata handling can process it
                        self.current -= 1;
                    }
                }
            }
            Opcode::VAArg => {
                // va_arg ptr_type ptr_val, result_type
                let _ptr_ty = self.parse_type()?;
                let _ptr_val = self.parse_value()?;
                self.consume(&Token::Comma)?;
                let result_ty = self.parse_type()?;
                result_type = Some(result_ty);  // va_arg result is the specified type
            }
            Opcode::Invoke => {
                // invoke [cc] [attrs] type @func(args...) to label %normal unwind label %exception
                self.skip_linkage_and_visibility();
                self.skip_attributes();

                let ret_ty = self.parse_type()?;
                result_type = Some(ret_ty);

                // Check for optional function signature
                if self.check(&Token::LParen) {
                    self.advance();
                    while !self.check(&Token::RParen) && !self.is_at_end() {
                        if self.match_token(&Token::Ellipsis) {
                            break;
                        }
                        self.parse_type()?;
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.consume(&Token::RParen)?;
                }

                // Parse function and arguments
                let func = self.parse_value()?;
                operands.push(func);
                self.consume(&Token::LParen)?;
                // For invoke, we don't know if it's varargs here, so assume false
                // TODO: Parse function type for invoke to detect varargs
                let args = self.parse_call_arguments_with_context(false)?;
                for (_, value) in args {
                    operands.push(value);
                }
                self.consume(&Token::RParen)?;

                // Parse destination labels
                self.consume(&Token::To)?;
                self.consume(&Token::Label)?;
                let normal_dest = self.expect_local_ident()?;
                operands.push(Value::new(
                    self.context.label_type(),
                    crate::value::ValueKind::BasicBlock,
                    Some(normal_dest)
                ));

                // Parse 'unwind' keyword (as identifier)
                if let Some(Token::Identifier(id)) = self.peek() {
                    if id == "unwind" {
                        self.advance();
                    }
                }
                self.consume(&Token::Label)?;
                let exception_dest = self.expect_local_ident()?;
                operands.push(Value::new(
                    self.context.label_type(),
                    crate::value::ValueKind::BasicBlock,
                    Some(exception_dest)
                ));

                // Handle optional operand bundles
                if self.check(&Token::LBracket) {
                    self.advance();
                    while !self.check(&Token::RBracket) && !self.is_at_end() {
                        if let Some(Token::StringLit(_)) = self.peek() {
                            self.advance();
                        }
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
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.match_token(&Token::RBracket);
                }

                self.skip_function_attributes();
            }
            Opcode::FNeg | Opcode::Freeze => {
                // Unary operations: fneg/freeze type value
                self.skip_instruction_flags();
                let ty = self.parse_type()?;
                result_type = Some(ty);
                let val = self.parse_value()?;
                operands.push(val);
            }
            Opcode::ShuffleVector => {
                // shufflevector <vec1>, <vec2>, <mask>
                let vec1_ty = self.parse_type()?;
                let vec1 = self.parse_value_with_type(Some(&vec1_ty))?;
                operands.push(vec1);
                self.consume(&Token::Comma)?;

                let vec2_ty = self.parse_type()?;
                let vec2 = self.parse_value_with_type(Some(&vec2_ty))?;
                operands.push(vec2);
                self.consume(&Token::Comma)?;

                let mask_ty = self.parse_type()?;
                let mask = self.parse_value_with_type(Some(&mask_ty))?;
                operands.push(mask);

                // Result type is same as first vector type
                result_type = Some(vec1_ty);
            }
            Opcode::ExtractValue => {
                // extractvalue <aggregate type> %agg, <idx>...
                let agg_ty = self.parse_type()?;
                let agg = self.parse_value_with_type(Some(&agg_ty))?;
                operands.push(agg);

                // Parse indices and compute result type
                let mut current_ty = agg_ty.clone();
                let mut indices = Vec::new();

                while self.match_token(&Token::Comma) {
                    if let Some(Token::Integer(idx)) = self.peek() {
                        let idx = *idx as usize;
                        indices.push(idx);

                        // Navigate to the indexed element type
                        if let Some(fields) = current_ty.struct_fields() {
                            if idx < fields.len() {
                                current_ty = fields[idx].clone();
                            }
                        } else if let Some((elem_ty, _size)) = current_ty.array_info() {
                            current_ty = elem_ty.clone();
                        }

                        let idx_val = Value::new(
                            self.context.int_type(32),
                            crate::value::ValueKind::ConstantInt { value: idx as i64 },
                            Some(idx.to_string())
                        );
                        operands.push(idx_val);
                        self.advance();
                    } else {
                        break;
                    }
                }

                // Result type is the final type after following all indices
                result_type = Some(current_ty);
            }
            Opcode::InsertValue => {
                // insertvalue <aggregate type> %agg, <element type> %val, <idx>...
                let agg_ty = self.parse_type()?;
                let agg = self.parse_value_with_type(Some(&agg_ty))?;
                operands.push(agg);
                self.consume(&Token::Comma)?;

                let elem_ty = self.parse_type()?;
                let elem = self.parse_value_with_type(Some(&elem_ty))?;
                operands.push(elem);

                // Parse indices
                while self.match_token(&Token::Comma) {
                    if let Some(Token::Integer(idx)) = self.peek() {
                        let idx_val = Value::new(
                            self.context.int_type(32),
                            crate::value::ValueKind::ConstantInt { value: *idx as i64 },
                            Some(idx.to_string())
                        );
                        operands.push(idx_val);
                        self.advance();
                    } else {
                        break;
                    }
                }

                // Result type is same as aggregate type
                result_type = Some(agg_ty);
            }
            Opcode::IndirectBr => {
                // indirectbr <ptr type> <address>, [ label <dest1>, label <dest2>, ... ]
                let addr_ty = self.parse_type()?;
                let addr = self.parse_value_with_type(Some(&addr_ty))?;
                operands.push(addr);
                self.consume(&Token::Comma)?;

                // Parse possible destinations
                self.consume(&Token::LBracket)?;
                while !self.check(&Token::RBracket) && !self.is_at_end() {
                    self.consume(&Token::Label)?;
                    let dest = self.expect_local_ident()?;
                    operands.push(Value::new(
                        self.context.label_type(),
                        crate::value::ValueKind::BasicBlock,
                        Some(dest)
                    ));

                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.consume(&Token::RBracket)?;
            }
            Opcode::Switch => {
                // switch <intty> <value>, label <defaultdest> [ <intty> <val>, label <dest> ]*
                // Parse condition type and value
                let cond_ty = self.parse_type()?;
                let cond_val = self.parse_value_with_type(Some(&cond_ty))?;
                operands.push(cond_val);

                self.consume(&Token::Comma)?;

                // Parse default destination: label %dest
                self.consume(&Token::Label)?;
                let default_dest = self.expect_local_ident()?;
                let default_label = Value::new(
                    self.context.label_type(),
                    crate::value::ValueKind::BasicBlock,
                    Some(default_dest)
                );
                operands.push(default_label);

                // Parse cases: [ <intty> <val>, label <dest> ... ]
                // Note: All cases are in a single bracket pair
                if self.match_token(&Token::LBracket) {
                    while !self.check(&Token::RBracket) && !self.is_at_end() {
                        // Parse case value: type value
                        let case_ty = self.parse_type()?;
                        let case_val = self.parse_value_with_type(Some(&case_ty))?;
                        operands.push(case_val);

                        self.consume(&Token::Comma)?;

                        // Parse case destination: label %dest
                        self.consume(&Token::Label)?;
                        let case_dest = self.expect_local_ident()?;
                        let case_label = Value::new(
                            self.context.label_type(),
                            crate::value::ValueKind::BasicBlock,
                            Some(case_dest)
                        );
                        operands.push(case_label);

                        // Cases are just whitespace/newline separated, no commas between them
                        // But there might be commas in some formats, so consume if present
                        self.match_token(&Token::Comma);
                    }
                    self.consume(&Token::RBracket)?;
                }
            }
            Opcode::Fence => {
                // fence [syncscope("<scope>")] <ordering>
                // Skip syncscope if present
                self.skip_syncscope();
                // Skip ordering (acquire, release, acq_rel, seq_cst)
                self.skip_atomic_ordering();
            }
            Opcode::Resume => {
                // resume type %value
                let ty = self.parse_type()?;
                let val = self.parse_value_with_type(Some(&ty))?;
                operands.push(val);
            }
            Opcode::LandingPad => {
                // landingpad type [cleanup] [catch/filter clauses...]
                // Parse result type
                result_type = Some(self.parse_type()?);

                // Skip cleanup if present
                if self.match_token(&Token::Cleanup) {
                    // cleanup flag
                }

                // Skip catch/filter clauses until next instruction
                let mut skip_count = 0;
                const MAX_SKIP: usize = 100;
                while !self.is_at_end() && skip_count < MAX_SKIP {
                    if self.check_local_ident() && self.peek_ahead(1) == Some(&Token::Equal) {
                        break;
                    }
                    if self.peek_ahead(1) == Some(&Token::Colon) {
                        break;
                    }
                    if self.check(&Token::RBrace) {
                        break;
                    }
                    self.advance();
                    skip_count += 1;
                }
            }
            Opcode::CatchPad => {
                // catchpad within %parentpad [args...]
                // Skip 'within' keyword if present
                while !self.is_at_end() && !self.check(&Token::LBracket) {
                    if self.check(&Token::RBrace) ||
                       (self.check_local_ident() && self.peek_ahead(1) == Some(&Token::Equal)) {
                        break;
                    }
                    self.advance();
                }
                // Skip args in brackets
                if self.match_token(&Token::LBracket) {
                    self.skip_to_matching_bracket();
                    self.advance();
                }
            }
            Opcode::CleanupPad => {
                // cleanuppad within %parentpad [args...]
                // Similar to catchpad
                while !self.is_at_end() && !self.check(&Token::LBracket) {
                    if self.check(&Token::RBrace) ||
                       (self.check_local_ident() && self.peek_ahead(1) == Some(&Token::Equal)) {
                        break;
                    }
                    self.advance();
                }
                if self.match_token(&Token::LBracket) {
                    self.skip_to_matching_bracket();
                    self.advance();
                }
            }
            Opcode::CatchSwitch => {
                // catchswitch within %parentpad [label %handler1, ...] unwind label %cleanup
                // Skip everything until next instruction
                let mut skip_count = 0;
                const MAX_SKIP: usize = 100;
                while !self.is_at_end() && skip_count < MAX_SKIP {
                    if self.check_local_ident() && self.peek_ahead(1) == Some(&Token::Equal) {
                        break;
                    }
                    if self.peek_ahead(1) == Some(&Token::Colon) {
                        break;
                    }
                    if self.check(&Token::RBrace) {
                        break;
                    }
                    self.advance();
                    skip_count += 1;
                }
            }
            Opcode::CleanupRet => {
                // cleanupret from %cleanuppad unwind label %continue / unwind to caller
                // Skip everything until next instruction
                let mut skip_count = 0;
                const MAX_SKIP: usize = 100;
                while !self.is_at_end() && skip_count < MAX_SKIP {
                    if self.check_local_ident() && self.peek_ahead(1) == Some(&Token::Equal) {
                        break;
                    }
                    if self.peek_ahead(1) == Some(&Token::Colon) {
                        break;
                    }
                    if self.check(&Token::RBrace) {
                        break;
                    }
                    self.advance();
                    skip_count += 1;
                }
            }
            Opcode::CatchRet => {
                // catchret from %catchpad to label %continue
                // Skip everything until next instruction
                let mut skip_count = 0;
                const MAX_SKIP: usize = 100;
                while !self.is_at_end() && skip_count < MAX_SKIP {
                    if self.check_local_ident() && self.peek_ahead(1) == Some(&Token::Equal) {
                        break;
                    }
                    if self.peek_ahead(1) == Some(&Token::Colon) {
                        break;
                    }
                    if self.check(&Token::RBrace) {
                        break;
                    }
                    self.advance();
                    skip_count += 1;
                }
            }
            // Binary operations: op [flags] type op1, type op2 or just op type op1, op2
            Opcode::Add | Opcode::FAdd | Opcode::Sub | Opcode::FSub | Opcode::Mul | Opcode::FMul |
            Opcode::UDiv | Opcode::SDiv | Opcode::FDiv | Opcode::URem | Opcode::SRem | Opcode::FRem |
            Opcode::Shl | Opcode::LShr | Opcode::AShr |
            Opcode::And | Opcode::Or | Opcode::Xor => {
                // Skip instruction flags (nsw, nuw, exact, fast-math flags, etc.)
                self.skip_instruction_flags();

                // Parse: type op1, op2
                let ty = self.parse_type()?;
                let op1 = self.parse_value_with_type(Some(&ty))?;
                operands.push(op1);
                self.consume(&Token::Comma)?;

                // Second operand might have explicit type or reuse first type
                // Check if next token is a type or a value
                let op2 = if self.check_type_token() {
                    let ty2 = self.parse_type()?;  // Usually same as ty
                    self.parse_value_with_type(Some(&ty2))?
                } else {
                    self.parse_value_with_type(Some(&ty))?
                };
                operands.push(op2);
                result_type = Some(ty);
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

        Ok((operands, result_type, gep_source_type_field, alignment, is_atomic))
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
           self.match_token(&Token::Une) || self.match_token(&Token::Ueq) ||
           self.match_token(&Token::True) ||
           self.match_token(&Token::False) {
            Ok(())
        } else {
            Err(ParseError::InvalidSyntax {
                message: "Expected comparison predicate".to_string(),
                position: self.current,
            })
        }
    }

    fn skip_metadata(&mut self) -> Option<String> {
        // Parse metadata reference and return the name if it's a named attachment
        // Returns: !{...}, !0, !DIExpression(), !foo, etc.
        // The lexer combines !foo into Token::MetadataIdent("foo"), so we need to handle both cases

        let metadata_name;

        // Case 1: Token::MetadataIdent - lexer already combined ! with identifier/number
        if let Some(Token::MetadataIdent(ref name)) = self.peek() {
            metadata_name = Some(name.clone());
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
            return metadata_name;
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
                return None;
            } else if let Some(Token::Identifier(ref name)) = self.peek() {
                // !DIExpression() - when lexer didn't combine them
                metadata_name = Some(name.clone());
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
                return metadata_name;
            } else if let Some(Token::Integer(_)) = self.peek() {
                // !0, !1, etc. - when lexer didn't combine them
                self.advance();
                return None;
            } else if let Some(Token::StringLit(_)) = self.peek() {
                // !"string" - metadata string literal
                self.advance();
                return None;
            }
        }
        None
    }

    /// Parse metadata node content and return actual Metadata object
    /// Handles: !{...}, !"string", !0, !DILocation(...), null
    fn parse_metadata_node(&mut self) -> ParseResult<crate::metadata::Metadata> {
        use crate::metadata::Metadata;

        // Case 1: MetadataIdent - reference to numbered or named metadata
        if let Some(Token::MetadataIdent(ref name)) = self.peek() {
            let md_name = name.clone();
            self.advance();

            // Check if it's a reference to existing metadata (!0, !1, etc.)
            if md_name.chars().all(|c| c.is_ascii_digit()) {
                // Reference to numbered metadata - look it up in registry
                if let Some(existing) = self.metadata_registry.get(&md_name) {
                    return Ok(existing.clone());
                } else {
                    // Forward reference - store reference to be resolved later
                    return Ok(Metadata::reference(md_name));
                }
            }

            // Named metadata like DILocation, DIExpression, etc.
            if self.check(&Token::LParen) {
                self.advance(); // consume (

                // Parse key: value pairs into a HashMap for field-based metadata
                let mut fields = std::collections::HashMap::new();
                let mut operands = Vec::new();
                let mut has_named_fields = false;

                while !self.check(&Token::RParen) && !self.is_at_end() {
                    // Check for key: value pairs or standalone identifiers
                    if let Some(Token::Identifier(field_name)) = self.peek() {
                        let field_name = field_name.clone();
                        self.advance(); // consume field name

                        if self.match_token(&Token::Colon) {
                            has_named_fields = true;
                            // Parse value
                            if let Some(Token::Integer(n)) = self.peek() {
                                fields.insert(field_name, Metadata::int(*n as i64));
                                operands.push(Metadata::int(*n as i64));
                                self.advance();
                            } else if let Some(Token::MetadataIdent(_)) = self.peek() {
                                // Recursive metadata reference
                                let inner = self.parse_metadata_node()?;
                                fields.insert(field_name, inner.clone());
                                operands.push(inner);
                            } else if let Some(Token::StringLit(s)) = self.peek() {
                                let s = s.clone();
                                fields.insert(field_name, Metadata::string(s.clone()));
                                operands.push(Metadata::string(s));
                                self.advance();
                            } else {
                                self.advance(); // skip unknown value
                            }
                        } else {
                            // Identifier without colon - treat as a string operand (e.g., DW_OP_swap)
                            operands.push(Metadata::string(field_name));
                        }
                    } else if let Some(Token::MetadataIdent(_)) = self.peek() {
                        // Positional metadata argument (no field name)
                        let inner = self.parse_metadata_node()?;
                        operands.push(inner);
                    } else if !self.check(&Token::Comma) && !self.check(&Token::RParen) {
                        // Unknown token - skip to avoid infinite loop
                        self.advance();
                    }

                    self.match_token(&Token::Comma);
                }

                self.consume(&Token::RParen)?;

                // If we found named fields, use NamedWithFields; otherwise use Named
                if has_named_fields && !fields.is_empty() {
                    return Ok(Metadata::named_with_fields(md_name, fields));
                } else {
                    return Ok(Metadata::named(md_name, operands));
                }
            }

            // Just a name without parens - return named metadata
            return Ok(Metadata::named(md_name, vec![]));
        }

        // Case 2: Exclaim followed by content
        if self.match_token(&Token::Exclaim) {
            // Case 2a: !{...} - tuple
            if self.check(&Token::LBrace) {
                self.advance(); // consume {
                let mut elements = Vec::new();

                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    // Parse each element
                    // Skip type annotations in metadata (e.g., "i32 1" -> parse just "1")
                    if self.check_type_token() {
                        self.parse_type().ok(); // Skip the type
                    }

                    if let Some(Token::MetadataIdent(_)) = self.peek() {
                        elements.push(self.parse_metadata_node()?);
                    } else if self.check(&Token::Exclaim) {
                        elements.push(self.parse_metadata_node()?);
                    } else if let Some(Token::Integer(n)) = self.peek() {
                        // Bare integer in metadata context
                        elements.push(Metadata::int(*n as i64));
                        self.advance();
                    } else if let Some(Token::StringLit(s)) = self.peek() {
                        elements.push(Metadata::string(s.clone()));
                        self.advance();
                    } else if self.match_token(&Token::Null) {
                        // null in metadata
                        elements.push(Metadata::tuple(vec![])); // Use empty tuple for null
                    } else {
                        // Skip unknown token
                        self.advance();
                    }

                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }

                self.consume(&Token::RBrace)?;
                return Ok(Metadata::tuple(elements));
            }

            // Case 2b: !"string" - string metadata
            if let Some(Token::StringLit(s)) = self.peek() {
                let string = s.clone();
                self.advance();
                return Ok(Metadata::string(string));
            }

            // Case 2c: !0, !1 - numbered reference
            if let Some(Token::Integer(n)) = self.peek() {
                let num = n.to_string();
                self.advance();
                // Look up in registry
                if let Some(existing) = self.metadata_registry.get(&num) {
                    return Ok(existing.clone());
                } else {
                    // Forward reference - store reference to be resolved later
                    return Ok(Metadata::reference(num));
                }
            }

            // Case 2d: !DILocation(...) when lexer didn't combine
            if let Some(Token::Identifier(name)) = self.peek() {
                let md_name = name.clone();
                self.advance();
                if self.check(&Token::LParen) {
                    // Put the identifier back and re-parse as MetadataIdent
                    self.current -= 1;
                    return self.parse_metadata_node();
                }
                return Ok(Metadata::named(md_name, vec![]));
            }
        }

        // Default: return empty tuple
        Ok(Metadata::tuple(vec![]))
    }

    fn parse_call_arguments_with_context(&mut self, is_varargs: bool) -> ParseResult<Vec<(Type, Value)>> {
        let mut args = Vec::new();

        while !self.check(&Token::RParen) && !self.is_at_end() {
            // Handle varargs ellipsis: call @f(ptr %x, ...)
            if self.match_token(&Token::Ellipsis) {
                break; // End of arguments
            }

            // Handle metadata arguments specially: metadata i32 0 or metadata !{}
            if self.match_token(&Token::Metadata) {
                // Check for various metadata forms
                let metadata_ty;
                let metadata_val;
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
                    metadata_ty = self.context.metadata_type();
                    metadata_val = Value::undef(metadata_ty.clone());
                } else if self.check(&Token::Exclaim) {
                    // metadata !{} - literal metadata
                    self.skip_metadata();
                    metadata_ty = self.context.metadata_type();
                    metadata_val = Value::undef(metadata_ty.clone());
                } else {
                    // metadata i32 0 - parse type and value
                    metadata_ty = self.context.metadata_type();
                    let _ty = self.parse_type()?;
                    let _val = self.parse_value()?;
                    metadata_val = Value::undef(metadata_ty.clone());
                }
                // Add metadata as an argument so verifier sees correct arg count
                args.push((metadata_ty, metadata_val));
                if !self.match_token(&Token::Comma) {
                    break;
                }
                continue;
            }

            let ty = self.parse_type()?;

            // Label types are not allowed as function arguments
            if ty.is_label() {
                return Err(ParseError::InvalidSyntax {
                    message: "invalid type for function argument".to_string(),
                    position: self.current,
                });
            }

            // Parse and validate parameter attributes (byval, sret, noundef, allocalign, etc.)
            let mut has_signext = false;
            let mut has_zeroext = false;
            let mut has_nest = false;
            let mut has_swifterror = false;
            let mut has_noalias = false;
            let mut has_align = false;
            let mut has_dereferenceable = false;
            let mut has_sret = false;

            loop {
                // Attributes without type parameters
                if self.match_token(&Token::Inreg) {
                    continue;
                }
                if self.match_token(&Token::Noalias) {
                    has_noalias = true;
                    continue;
                }
                if self.match_token(&Token::Nocapture) {
                    continue;
                }
                if self.match_token(&Token::Nest) {
                    has_nest = true;
                    continue;
                }
                if self.match_token(&Token::Zeroext) {
                    has_zeroext = true;
                    continue;
                }
                if self.match_token(&Token::Signext) {
                    has_signext = true;
                    continue;
                }
                if self.match_token(&Token::Immarg) {
                    continue;
                }
                if self.match_token(&Token::Nonnull) {
                    continue;
                }
                if self.match_token(&Token::Readonly) {
                    continue;
                }
                if self.match_token(&Token::Writeonly) {
                    continue;
                }
                if self.match_token(&Token::Swifterror) {
                    has_swifterror = true;
                    continue;
                }
                if self.match_token(&Token::Swiftself) {
                    continue;
                }
                if self.match_token(&Token::Swiftasync) {
                    continue;
                }

                // Attributes with parameters: dereferenceable(N), dereferenceable_or_null(N)
                if self.match_token(&Token::Dereferenceable) ||
                   self.match_token(&Token::Dereferenceable_or_null) {
                    has_dereferenceable = true;
                    if self.check(&Token::LParen) {
                        self.advance();
                        while !self.check(&Token::RParen) && !self.is_at_end() {
                            self.advance();
                        }
                        self.match_token(&Token::RParen);
                    }
                    continue;
                }

                // Attributes with optional type parameters: byval(type), sret(type), inalloca(type), preallocated(type)
                if self.match_token(&Token::Byval) {
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
                if self.match_token(&Token::Sret) {
                    has_sret = true;
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
                if self.match_token(&Token::Inalloca) ||
                   self.match_token(&Token::Preallocated) {
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

                // Handle identifier-based attributes with type parameters: byref(type), elementtype(type), nofpclass(...), preallocated(type), range(type low, high)
                if let Some(Token::Identifier(attr)) = self.peek() {
                    if matches!(attr.as_str(), "byref" | "elementtype" | "nofpclass" | "preallocated" | "range") {
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
                    has_align = true;
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

            // Validate attributes against type
            // signext/zeroext must be on integer types
            if has_signext && !ty.is_integer() {
                return Err(ParseError::InvalidAttribute {
                    message: "Attribute 'signext' applied to incompatible type!".to_string(),
                });
            }
            if has_zeroext && !ty.is_integer() {
                return Err(ParseError::InvalidAttribute {
                    message: "Attribute 'zeroext' applied to incompatible type!".to_string(),
                });
            }
            // nest, swifterror, noalias, align, dereferenceable must be on pointer types
            if has_nest && !ty.is_pointer() {
                return Err(ParseError::InvalidAttribute {
                    message: "Attribute 'nest' applied to incompatible type!".to_string(),
                });
            }
            if has_swifterror && !ty.is_pointer() {
                return Err(ParseError::InvalidAttribute {
                    message: "Attribute 'swifterror' applied to incompatible type!".to_string(),
                });
            }
            if has_noalias && !ty.is_pointer() {
                return Err(ParseError::InvalidAttribute {
                    message: "Attribute 'noalias' applied to incompatible type!".to_string(),
                });
            }
            if has_align && !ty.is_pointer() {
                return Err(ParseError::InvalidAttribute {
                    message: "Attribute 'align' applied to incompatible type!".to_string(),
                });
            }
            if has_dereferenceable && !ty.is_pointer() {
                return Err(ParseError::InvalidAttribute {
                    message: "Attribute 'dereferenceable' applied to incompatible type!".to_string(),
                });
            }
            // sret cannot be used in varargs functions
            if has_sret && is_varargs {
                return Err(ParseError::InvalidAttribute {
                    message: "Attribute 'sret' cannot be used in a varargs function call".to_string(),
                });
            }

            let val = self.parse_value_with_type(Some(&ty))?;
            args.push((ty.clone(), val));

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
            Token::X86_mmx => {
                // x86 matrix/vector types
                self.advance();
                Ok(self.context.void_type()) // Placeholder for x86_mmx type
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
                    let address_space = if self.check(&Token::Addrspace) {
                        self.advance(); // consume 'addrspace'
                        self.consume(&Token::LParen)?;
                        // Parse address space number
                        let addrspace = if let Some(Token::Integer(n)) = self.peek() {
                            // Address space must fit in 24 bits: max value is 16,777,215 (2^24 - 1)
                            if *n < 0 || *n >= (1 << 24) {
                                return Err(ParseError::InvalidSyntax {
                                    message: "invalid address space, must be a 24-bit integer".to_string(),
                                    position: self.current,
                                });
                            }
                            let val = *n as u32;
                            self.advance();
                            val
                        } else if let Some(Token::StringLit(s)) = self.peek() {
                            // Symbolic address space: map to number
                            let val = match s.as_str() {
                                "A" => 1,
                                "G" => 2,
                                "P" => 3,
                                _ => 0,
                            };
                            self.advance();
                            val
                        } else {
                            // Invalid token or empty - skip gracefully if present
                            if !self.check(&Token::RParen) {
                                self.advance();
                            }
                            0
                        };
                        self.consume(&Token::RParen)?;
                        addrspace
                    } else {
                        0
                    };
                    // Modern LLVM uses opaque pointers (ptr)
                    Ok(Type::ptr_addrspace(&self.context, self.context.int8_type(), address_space))
                }
            }
            Token::Label => {
                self.advance();
                Ok(self.context.label_type())
            }
            Token::Token => {
                self.advance();
                // Token type for statepoints/gc
                Ok(self.context.token_type())
            }
            Token::Metadata => {
                self.advance();
                Ok(self.context.metadata_type())
            }
            Token::X86_amx => {
                self.advance();
                Ok(self.context.x86_amx_type())
            }
            Token::Target => {
                // target("typename", params...) - target-specific types with optional type/integer params
                self.advance(); // consume 'target'
                self.consume(&Token::LParen)?;
                let type_name = if let Some(Token::StringLit(name)) = self.peek() {
                    let name = name.clone();
                    self.advance(); // consume type name string
                    name
                } else {
                    "unknown".to_string()
                };
                // Handle optional comma-separated parameters (types or integers)
                while self.match_token(&Token::Comma) {
                    // Parameter can be a type or an integer
                    if let Some(Token::Integer(_)) = self.peek() {
                        self.advance(); // consume integer parameter
                    } else {
                        // Try to parse as type
                        let _ = self.parse_type();
                    }
                }
                self.consume(&Token::RParen)?;
                // Return opaque type for target-specific types
                Ok(Type::opaque(&self.context, format!("target({})", type_name)))
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
                // Could be:
                // 1. Vector type: < size x type > or scalable: < vscale x size x type >
                // 2. Packed struct: <{ type1, type2, ... }>
                self.advance();

                // Check if this is a packed struct <{...}>
                if self.match_token(&Token::LBrace) {
                    // Packed struct
                    let mut field_types = Vec::new();
                    // Handle empty packed struct <{}>
                    if !self.check(&Token::RBrace) {
                        loop {
                            field_types.push(self.parse_type()?);
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(&Token::RBrace)?;
                    self.consume(&Token::RAngle)?;
                    Ok(crate::types::Type::struct_type_packed(&self.context, field_types, None, true))
                } else {
                    // Vector type
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
            }
            Token::LocalIdent(name) => {
                // Type reference like %TypeName or %0
                let name = name.clone();
                self.advance();
                // Look up in type table
                if let Some(ty) = self.type_table.get(&name) {
                    Ok(ty.clone())
                } else {
                    // Type not found - treat as opaque type placeholder (use i8 to ensure it's sized for alloca)
                    Ok(self.context.int8_type())
                }
            }
            Token::Ellipsis => {
                // Ellipsis for varargs - should be caught before parse_type() is called
                // This is a defensive case that shouldn't normally be reached
                self.advance();
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
            let mut is_vararg = false;
            while !self.check(&Token::RParen) && !self.is_at_end() {
                // Check for varargs
                if self.check(&Token::Ellipsis) {
                    self.advance();
                    is_vararg = true;
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
            let mut func_type = self.context.function_type(base_type, param_types, is_vararg);

            // Check for stars to make function pointer: void ()* or void ()**
            while self.check(&Token::Star) {
                self.advance();
                func_type = self.context.ptr_type(func_type);
            }

            return Ok(func_type);
        }

        // Check for old-style typed pointer syntax: type addrspace(n)* or type*
        // Handle patterns like: i8*, i8 addrspace(4)*, i8 addrspace(4)* addrspace(4)*
        let mut result_type = base_type;
        loop {
            // Parse optional addrspace modifier before each star
            let address_space = if self.check(&Token::Addrspace) {
                self.advance(); // consume 'addrspace'
                self.consume(&Token::LParen)?;
                let addrspace = if let Some(Token::Integer(n)) = self.peek() {
                    // Address space must fit in 24 bits: max value is 16,777,215 (2^24 - 1)
                    if *n < 0 || *n >= (1 << 24) {
                        return Err(ParseError::InvalidSyntax {
                            message: "invalid address space, must be a 24-bit integer".to_string(),
                            position: self.current,
                        });
                    }
                    let val = *n as u32;
                    self.advance();
                    val
                } else if let Some(Token::StringLit(s)) = self.peek() {
                    // Symbolic address space: map to number
                    let val = match s.as_str() {
                        "A" => 1,
                        "G" => 2,
                        "P" => 3,
                        _ => 0,
                    };
                    self.advance();
                    val
                } else {
                    // Invalid token or empty - skip gracefully if present
                    if !self.check(&Token::RParen) {
                        self.advance();
                    }
                    0
                };
                self.consume(&Token::RParen)?;
                addrspace
            } else {
                0
            };

            // Check for * to make it a pointer
            if self.check(&Token::Star) {
                self.advance(); // consume '*'
                result_type = Type::ptr_addrspace(&self.context, result_type, address_space);
            } else {
                break; // No more stars, we're done
            }
        }

        Ok(result_type)
    }

    fn parse_value(&mut self) -> ParseResult<Value> {
        self.parse_value_with_type(None)
    }

    fn parse_value_with_type(&mut self, expected_type: Option<&Type>) -> ParseResult<Value> {
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
                // Expected type should be a vector type like <4 x i32>
                self.advance(); // consume 'splat'
                self.consume(&Token::LParen)?;
                let elem_ty = self.parse_type()?;
                let elem_val = self.parse_value_with_type(Some(&elem_ty))?;
                self.consume(&Token::RParen)?;

                // Use expected_type to determine vector size
                if let Some(vec_ty) = expected_type {
                    if vec_ty.is_vector() {
                        // Create vector constant with all elements set to elem_val
                        // For now, return a vector constant (implementation depends on Value API)
                        Ok(Value::vector_splat(vec_ty.clone(), elem_val))
                    } else {
                        // Expected type is not a vector - return the element value
                        Ok(elem_val)
                    }
                } else {
                    // No expected type - use void as fallback (should not happen in valid IR)
                    Ok(Value::zero_initializer(self.context.void_type()))
                }
            }
            Token::Identifier(id) if id == "dso_local_equivalent" => {
                // DSO local equivalent: dso_local_equivalent @func
                self.advance(); // consume 'dso_local_equivalent'
                let _val = self.parse_value()?; // Parse the wrapped value
                // Return placeholder - treat as the wrapped value
                Ok(Value::zero_initializer(self.context.void_type()))
            }
            Token::Identifier(id) if id == "no_cfi" => {
                // no_cfi marker: no_cfi @func
                self.advance(); // consume 'no_cfi'
                let _val = self.parse_value()?; // Parse the wrapped value
                // Return placeholder - treat as the wrapped value
                Ok(Value::zero_initializer(self.context.void_type()))
            }
            Token::Identifier(id) if id == "blockaddress" => {
                // Block address: blockaddress(@func, %block)
                // Returns a pointer to a basic block
                self.advance(); // consume 'blockaddress'
                self.consume(&Token::LParen)?;
                let func = self.parse_value()?; // @func
                self.consume(&Token::Comma)?;
                let block = self.parse_value()?; // %block
                self.consume(&Token::RParen)?;
                let ty = expected_type.cloned().unwrap_or_else(|| self.context.ptr_type(self.context.int8_type()));
                Ok(Value::block_address(ty, func, block))
            }
            Token::LocalIdent(name) => {
                let name = name.clone();
                self.advance();
                // Look up in symbol table first
                if let Some(value) = self.symbol_table.get(&name) {
                    Ok(value.clone())
                } else {
                    // If not found, create a placeholder instruction value for local variables
                    // This can happen for forward references (e.g., phi nodes that reference themselves)
                    // Use expected_type if provided, otherwise default to void
                    let ty = expected_type.cloned().unwrap_or_else(|| self.context.void_type());
                    Ok(Value::instruction(ty, Opcode::Add, Some(name)))
                }
            }
            Token::GlobalIdent(name) => {
                let name = name.clone();
                self.advance();
                // Use expected type if provided, otherwise look up function declaration, otherwise default to ptr
                let ty = if let Some(expected) = expected_type {
                    expected.clone()
                } else if let Some(fn_type) = self.function_decls.get(&name) {
                    // Function references are pointers to functions
                    if fn_type.is_function() {
                        self.context.ptr_type(fn_type.clone())
                    } else {
                        fn_type.clone()
                    }
                } else {
                    // Global variable reference - default to ptr in opaque pointer mode
                    self.context.ptr_type(self.context.int8_type())
                };
                Ok(Value::new(ty, crate::value::ValueKind::GlobalVariable { is_constant: false }, Some(name)))
            }
            Token::Integer(n) => {
                let n = *n;
                self.advance();
                // Check if expected type is float - hex integer constants can be float representations
                if let Some(expected) = expected_type {
                    if expected.is_float() {
                        // Hex integer constant used as float (e.g., 0x427F4000 for double)
                        // Convert the hex bits to a float based on the float size
                        // Check the expected type's format string to determine float width
                        let expected_str = format!("{}", expected);
                        let float_val = if expected_str == "double" || expected_str.starts_with("fp128") {
                            // 64-bit double from i64 bits
                            f64::from_bits(n as u64)
                        } else {
                            // 32-bit float or half from i32 bits
                            f32::from_bits(n as u32) as f64
                        };
                        return Ok(Value::const_float(expected.clone(), float_val, None));
                    } else if expected.is_integer() {
                        return Ok(Value::const_int(expected.clone(), n as i64, None));
                    }
                }
                // Default: integer type
                Ok(Value::const_int(self.context.int32_type(), n as i64, None))
            }
            Token::Float64(f) => {
                let f = *f;
                self.advance();
                // Use expected type if provided and it's a floating point type
                let ty = if let Some(expected) = expected_type {
                    if expected.is_float() {
                        expected.clone()
                    } else {
                        self.context.double_type()
                    }
                } else {
                    self.context.double_type()
                };
                Ok(Value::const_float(ty, f, None))
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
            Token::CString(bytes) => {
                let bytes = bytes.clone();
                self.advance();
                // CString like c"foo\00" - create array of i8 constant
                let i8_type = self.context.int8_type();
                let array_type = self.context.array_type(i8_type, bytes.len());
                // For now, create a zero initializer (full implementation would create proper const array)
                Ok(Value::zero_initializer(array_type))
            }
            Token::None => {
                self.advance();
                // 'none' is used with token type in GC intrinsics
                Ok(Value::undef(self.context.token_type()))
            }
            Token::Undef => {
                self.advance();
                let ty = expected_type.cloned().unwrap_or_else(|| self.context.void_type());
                Ok(Value::undef(ty))
            }
            Token::Poison => {
                self.advance();
                let ty = expected_type.cloned().unwrap_or_else(|| self.context.void_type());
                Ok(Value::undef(ty)) // Treat poison like undef for now
            }
            Token::Zeroinitializer => {
                self.advance();
                let ty = expected_type.cloned().unwrap_or_else(|| self.context.void_type());
                Ok(Value::zero_initializer(ty))
            }
            Token::LAngle => {
                // Could be:
                // 1. Vector constant: < type val1, type val2, ... >
                // 2. Packed struct constant: <{ type val1, type val2, ... }>
                self.advance(); // consume '<'

                // Check for packed struct constant
                if self.match_token(&Token::LBrace) {
                    // Packed struct constant: <{ type val, type val, ... }>
                    // Handle empty packed struct constant <{}>
                    if !self.check(&Token::RBrace) {
                        loop {
                            let _ty = self.parse_type()?;
                            let _val = self.parse_value()?;
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(&Token::RBrace)?;
                    self.consume(&Token::RAngle)?;
                    // Return placeholder packed struct constant
                    let ty = expected_type.cloned().unwrap_or_else(|| self.context.void_type());
                    Ok(Value::zero_initializer(ty))
                } else {
                    // Vector constant
                    while !self.check(&Token::RAngle) && !self.is_at_end() {
                        // Parse element type and value
                        let _elem_ty = self.parse_type()?;
                        let _elem_val = self.parse_value()?;
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.consume(&Token::RAngle)?;
                    // Return placeholder vector constant with expected type
                    let ty = expected_type.cloned().unwrap_or_else(|| self.context.void_type());
                    Ok(Value::zero_initializer(ty))
                }
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
                // Return placeholder array constant with expected type
                let ty = expected_type.cloned().unwrap_or_else(|| self.context.void_type());
                Ok(Value::zero_initializer(ty))
            }
            Token::LBrace => {
                // Struct constant: { type val1, type val2, ... }
                self.advance(); // consume '{'

                // Get expected field types if we have a struct expected type
                let expected_fields = if let Some(expected_ty) = expected_type {
                    if expected_ty.is_struct() {
                        expected_ty.struct_fields()
                    } else {
                        None
                    }
                } else {
                    None
                };

                let mut field_values = Vec::new();
                let mut field_index = 0;

                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    let elem_ty = self.parse_type()?;
                    let elem_val = self.parse_value_with_type(Some(&elem_ty))?;

                    // Validate that value type matches declared type in initializer
                    if elem_val.get_type() != &elem_ty {
                        return Err(ParseError::InvalidSyntax {
                            message: format!(
                                "initializer value type mismatch: declared {:?}, got {:?}",
                                elem_ty, elem_val.get_type()
                            ),
                            position: self.current,
                        });
                    }

                    // Validate against expected struct field type if we have one
                    if let Some(ref fields) = expected_fields {
                        if field_index >= fields.len() {
                            return Err(ParseError::InvalidSyntax {
                                message: format!(
                                    "too many elements in struct initializer: struct has {} fields",
                                    fields.len()
                                ),
                                position: self.current,
                            });
                        }

                        if elem_val.get_type() != &fields[field_index] {
                            return Err(ParseError::InvalidSyntax {
                                message: format!(
                                    "struct initializer doesn't match struct element type: field {} expected {:?}, got {:?}",
                                    field_index, fields[field_index], elem_val.get_type()
                                ),
                                position: self.current,
                            });
                        }
                    }

                    field_values.push(elem_val);
                    field_index += 1;

                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
                self.consume(&Token::RBrace)?;

                // Final validation: check we got enough elements
                if let Some(ref fields) = expected_fields {
                    if field_values.len() != fields.len() {
                        return Err(ParseError::InvalidSyntax {
                            message: format!(
                                "initializer with struct type has wrong # elements: expected {}, got {}",
                                fields.len(), field_values.len()
                            ),
                            position: self.current,
                        });
                    }
                }

                // Return the appropriate value
                if let Some(expected_ty) = expected_type {
                    if expected_ty.is_struct() {
                        Ok(Value::const_struct(expected_ty.clone(), field_values))
                    } else {
                        Ok(Value::zero_initializer(expected_ty.clone()))
                    }
                } else {
                    // No expected type - create anonymous struct from field values
                    let field_types: Vec<Type> = field_values.iter().map(|v| v.get_type().clone()).collect();
                    let struct_ty = Type::struct_type(&self.context, field_types, None);
                    Ok(Value::const_struct(struct_ty, field_values))
                }
            }
            // Constant expressions - instructions that can appear in constant contexts
            Token::PtrToInt | Token::IntToPtr | Token::PtrToAddr | Token::AddrToPtr |
            Token::BitCast | Token::AddrSpaceCast |
            Token::Trunc | Token::ZExt | Token::SExt | Token::FPTrunc | Token::FPExt |
            Token::FPToUI | Token::FPToSI | Token::UIToFP | Token::SIToFP |
            Token::GetElementPtr | Token::Sub | Token::Add | Token::Mul | Token::FNeg |
            Token::UDiv | Token::SDiv | Token::URem | Token::SRem |
            Token::Shl | Token::LShr | Token::AShr | Token::And | Token::Or | Token::Xor |
            Token::ICmp | Token::FCmp | Token::Select | Token::ExtractValue |
            Token::ExtractElement | Token::InsertElement | Token::ShuffleVector => {
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
        // Parse constant expressions like: ptrtoint (ptr @global to i32), inttoptr (i64 50 to ptr)
        let token = self.peek().ok_or(ParseError::UnexpectedEOF)?;

        // Determine the opcode
        let opcode = match token {
            Token::PtrToInt => Opcode::PtrToInt,
            Token::IntToPtr => Opcode::IntToPtr,
            Token::PtrToAddr => Opcode::PtrToAddr,
            Token::AddrToPtr => Opcode::AddrToPtr,
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
            Token::FNeg => Opcode::FNeg,
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
            Token::ExtractValue => Opcode::ExtractValue,
            Token::ExtractElement => Opcode::ExtractElement,
            Token::InsertElement => Opcode::InsertElement,
            Token::ShuffleVector => Opcode::ShuffleVector,
            _ => {
                return Err(ParseError::InvalidSyntax {
                    message: format!("Unexpected token in constant expression: {:?}", token),
                    position: self.current,
                });
            }
        };

        self.advance(); // consume opcode token

        // Skip instruction flags (nuw, nsw, exact, fast-math flags)
        self.skip_instruction_flags();

        // For ICmp/FCmp, parse the predicate before the opening paren: icmp ne (...)
        if matches!(opcode, Opcode::ICmp | Opcode::FCmp) {
            self.parse_comparison_predicate()?;
        }

        // Parse the operands inside parentheses
        self.consume(&Token::LParen)?;

        // For cast operations: castop (srctype value to desttype)
        // For binary ops: binop (type val1, type val2)
        // For GEP: getelementptr (basetype, ptrtype ptrvalue, indices...)
        // For select: select (type cond, type val1, type val2)

        let mut result_type: Option<Type> = None;

        if matches!(opcode, Opcode::GetElementPtr) {
            // GEP is special: getelementptr (basetype, ptrtype ptrvalue, indextype indexvalue, ...)
            let _base_ty = self.parse_type()?;
            self.consume(&Token::Comma)?;
            let ptr_ty = self.parse_type()?;
            let _ptr_val = self.parse_value()?;

            // Determine result type: ptr or <N x ptr> if any index is a vector
            let mut vector_size = None;

            // Check if base is a vector
            if ptr_ty.is_vector() {
                vector_size = ptr_ty.vector_info().map(|(_, size)| size);
            }

            // Parse remaining indices and check for vectors
            while self.match_token(&Token::Comma) {
                self.match_token(&Token::Inrange); // Skip optional inrange
                let idx_ty = self.parse_type()?;

                // Check if this index is a vector
                if idx_ty.is_vector() {
                    vector_size = idx_ty.vector_info().map(|(_, size)| size);
                }

                let _idx_val = self.parse_value()?;
            }

            // Set result type based on whether we found a vector
            result_type = if let Some(size) = vector_size {
                Some(self.context.vector_type(
                    self.context.ptr_type(self.context.int8_type()),
                    size
                ))
            } else {
                Some(self.context.ptr_type(self.context.int8_type()))
            };
        } else {
            // Simplified parsing - just parse type and value, skip to closing paren
            // This allows the constant expression to be recognized without full semantic support
            let src_ty = self.parse_type()?;
            let _src_val = self.parse_value()?;

            // Handle 'to' keyword for casts - destination type is the result type
            if matches!(opcode, Opcode::PtrToInt | Opcode::IntToPtr | Opcode::PtrToAddr | Opcode::AddrToPtr |
                               Opcode::BitCast | Opcode::Trunc | Opcode::ZExt | Opcode::SExt |
                               Opcode::FPTrunc | Opcode::FPExt | Opcode::FPToUI |
                               Opcode::FPToSI | Opcode::UIToFP | Opcode::SIToFP |
                               Opcode::AddrSpaceCast) {
                if self.match_token(&Token::To) {
                    let dest_ty = self.parse_type()?;
                    result_type = Some(dest_ty);  // Cast result is destination type
                }
            } else if matches!(opcode, Opcode::ICmp | Opcode::FCmp) {
                // Comparison results are always i1
                result_type = Some(self.context.int_type(1));
            } else if matches!(opcode, Opcode::Select) {
                // Select: select (type cond, type val1, type val2)
                // Result type is the value type (second argument)
                if self.match_token(&Token::Comma) {
                    let val_ty = self.parse_type()?;
                    result_type = Some(val_ty);  // Select result is value type
                    let _val2 = self.parse_value()?;
                    if self.match_token(&Token::Comma) {
                        let _ty3 = self.parse_type()?;
                        let _val3 = self.parse_value()?;
                    }
                }
            } else if matches!(opcode, Opcode::ShuffleVector) {
                // ShuffleVector has 3 operands: shufflevector (type vec1, type vec2, type mask)
                // Result type is the first vector type
                result_type = Some(src_ty.clone());
                if self.match_token(&Token::Comma) {
                    let _ty2 = self.parse_type()?;
                    let _val2 = self.parse_value()?;
                    if self.match_token(&Token::Comma) {
                        let _ty3 = self.parse_type()?;
                        let _val3 = self.parse_value()?;
                    }
                }
            } else if matches!(opcode, Opcode::InsertElement) {
                // InsertElement has 3 operands: insertelement (type vec, type val, type idx)
                // Result type is the vector type
                result_type = Some(src_ty.clone());
                if self.match_token(&Token::Comma) {
                    let _ty2 = self.parse_type()?;
                    let _val2 = self.parse_value()?;
                    if self.match_token(&Token::Comma) {
                        let _ty3 = self.parse_type()?;
                        let _val3 = self.parse_value()?;
                    }
                }
            } else {
                // Binary operations and others - result type is operand type
                result_type = Some(src_ty.clone());
                // Parse second operand if comma present
                if self.match_token(&Token::Comma) {
                    let _ty2 = self.parse_type()?;
                    let _val2 = self.parse_value()?;
                }
            }
        }

        self.consume(&Token::RParen)?;

        // Return a constant expression value with the correct result type
        let ty = result_type.unwrap_or_else(|| self.context.void_type());
        Ok(Value::instruction(ty, opcode, Some("constexpr".to_string())))
    }

    fn parse_parameters(&mut self) -> ParseResult<(Vec<(Type, String)>, Vec<crate::function::ParameterAttributes>, bool)> {
        let mut params = Vec::new();
        let mut param_attrs = Vec::new();
        let mut is_vararg = false;

        while !self.check(&Token::RParen) && !self.is_at_end() {
            // Check for varargs (just ellipsis with no type)
            if self.check(&Token::Ellipsis) {
                self.advance();
                is_vararg = true;
                break;
            }

            let ty = self.parse_type()?;

            // Parse parameter attributes
            let attrs = self.parse_parameter_attributes();

            let name = if let Some(Token::LocalIdent(n)) = self.peek().cloned() {
                self.advance();
                n
            } else {
                format!("arg{}", params.len())
            };

            params.push((ty, name));
            param_attrs.push(attrs);

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok((params, param_attrs, is_vararg))
    }

    fn parse_parameter_types(&mut self) -> ParseResult<(Vec<Type>, Vec<crate::function::ParameterAttributes>, bool)> {
        let mut types = Vec::new();
        let mut param_attrs = Vec::new();
        let mut is_vararg = false;

        while !self.check(&Token::RParen) && !self.is_at_end() {
            // Check for varargs
            if self.check(&Token::Ellipsis) {
                self.advance();
                is_vararg = true;
                break;
            }

            let ty = self.parse_type()?;
            types.push(ty);

            // Parse parameter attributes
            let attrs = self.parse_parameter_attributes();
            param_attrs.push(attrs);

            // Skip local ident if present
            if self.check_local_ident() {
                self.advance();
            }

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok((types, param_attrs, is_vararg))
    }

    // Helper methods

    fn parse_calling_convention(&mut self) -> CallingConvention {
        let mut cc = CallingConvention::C;

        loop {
            // Check token-based calling conventions first
            if self.match_token(&Token::Amdgpu_kernel) {
                cc = CallingConvention::AMDGPU_Kernel;
                continue;
            }
            if self.match_token(&Token::Amdgpu_ps) {
                cc = CallingConvention::AMDGPU_PS;
                continue;
            }
            if self.match_token(&Token::Amdgpu_cs_chain) {
                cc = CallingConvention::AMDGPU_CS_Chain;
                continue;
            }

            // Check identifier-based calling conventions
            if let Some(Token::Identifier(id)) = self.peek() {
                let cc_name = id.clone();
                let cc_opt = match cc_name.as_str() {
                    "ccc" => Some(CallingConvention::C),
                    "fastcc" => Some(CallingConvention::Fast),
                    "coldcc" => Some(CallingConvention::Cold),
                    "tailcc" => Some(CallingConvention::Tail),
                    "swiftcc" => Some(CallingConvention::Swift),
                    "swifttailcc" => Some(CallingConvention::SwiftTail),
                    "amdgpu_kernel" => Some(CallingConvention::AMDGPU_Kernel),
                    "amdgpu_vs" => Some(CallingConvention::AMDGPU_VS),
                    "amdgpu_gs" => Some(CallingConvention::AMDGPU_GS),
                    "amdgpu_ps" => Some(CallingConvention::AMDGPU_PS),
                    "amdgpu_cs" => Some(CallingConvention::AMDGPU_CS),
                    "amdgpu_hs" => Some(CallingConvention::AMDGPU_HS),
                    "amdgpu_ls" => Some(CallingConvention::AMDGPU_LS),
                    "amdgpu_es" => Some(CallingConvention::AMDGPU_ES),
                    "amdgpu_cs_chain" => Some(CallingConvention::AMDGPU_CS_Chain),
                    "amdgpu_cs_chain_preserve" => Some(CallingConvention::AMDGPU_CS_Chain_Preserve),
                    "amdgpu_gfx_whole_wave" => Some(CallingConvention::AMDGPU_GFX_Whole_Wave),
                    "spir_kernel" => Some(CallingConvention::SPIR_Kernel),
                    "spir_func" => Some(CallingConvention::SPIR_Func),
                    "x86_stdcallcc" => Some(CallingConvention::X86_StdCall),
                    _ if cc_name.starts_with("amdgpu_") || cc_name.starts_with("spir_") || cc_name.starts_with("x86_") => {
                        // Unknown variant, skip it
                        None
                    },
                    _ => None,
                };

                if cc_opt.is_some() || cc_name.starts_with("amdgpu_") || cc_name.starts_with("spir_") || cc_name.starts_with("x86_") {
                    if let Some(c) = cc_opt {
                        cc = c;
                    }
                    self.advance();
                    // Some calling conventions have parameters
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

            break;
        }

        // Now skip linkage/visibility keywords
        self.skip_linkage_and_visibility();
        cc
    }

    fn parse_linkage(&mut self) -> crate::module::Linkage {
        use crate::module::Linkage;

        let linkage = if self.match_token(&Token::Private) {
            Linkage::Private
        } else if self.match_token(&Token::Internal) {
            Linkage::Internal
        } else if self.match_token(&Token::External) {
            Linkage::External
        } else if self.match_token(&Token::Weak) {
            Linkage::Weak
        } else if self.match_token(&Token::Weak_odr) {
            Linkage::WeakOdr
        } else if self.match_token(&Token::Linkonce) {
            Linkage::Linkonce
        } else if self.match_token(&Token::Linkonce_odr) {
            Linkage::LinkonceOdr
        } else if self.match_token(&Token::Available_externally) {
            Linkage::AvailableExternally
        } else if self.match_token(&Token::Extern_weak) {
            Linkage::ExternWeak
        } else if self.match_token(&Token::Common) {
            Linkage::Common
        } else if self.match_token(&Token::Appending) {
            Linkage::Appending
        } else {
            Linkage::External // default
        };

        linkage
    }

    fn parse_visibility(&mut self) -> crate::module::Visibility {
        use crate::module::Visibility;

        let visibility = if self.match_token(&Token::Default) {
            Visibility::Default
        } else if self.match_token(&Token::Hidden) {
            Visibility::Hidden
        } else if self.match_token(&Token::Protected) {
            Visibility::Protected
        } else {
            Visibility::Default // default
        };

        visibility
    }

    fn parse_dll_storage_class(&mut self) -> crate::module::DLLStorageClass {
        use crate::module::DLLStorageClass;

        if self.match_token(&Token::Dllimport) {
            DLLStorageClass::DllImport
        } else if self.match_token(&Token::Dllexport) {
            DLLStorageClass::DllExport
        } else {
            DLLStorageClass::Default
        }
    }

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
               self.match_token(&Token::Local_unnamed_addr) ||
               // GPU calling conventions
               self.match_token(&Token::Amdgpu_kernel) ||
               self.match_token(&Token::Amdgpu_cs_chain) ||
               self.match_token(&Token::Amdgpu_ps) {
                // Keep consuming
                continue;
            }

            // Thread_local with optional parameters: thread_local(localdynamic)
            if self.match_token(&Token::Thread_local) {
                if self.check(&Token::LParen) {
                    self.advance(); // consume (
                    while !self.check(&Token::RParen) && !self.is_at_end() {
                        self.advance();
                    }
                    self.match_token(&Token::RParen); // consume )
                }
                continue;
            }

            // Check for identifier-based calling conventions (e.g., amdgpu_cs_chain_preserve, x86_intrcc, riscv_vls_cc)
            if let Some(Token::Identifier(id)) = self.peek() {
                if id.starts_with("amdgpu_") || id.starts_with("spir_") ||
                   id.starts_with("aarch64_") || id.starts_with("x86_") ||
                   id.starts_with("riscv_") || id.starts_with("arm_") ||
                   id.starts_with("avr_") || id.starts_with("ptx_") ||
                   id.starts_with("msp430_") || id.starts_with("preserve_") ||
                   id == "cc" || id.starts_with("cc") ||
                   id == "ccc" || id == "fastcc" || id == "coldcc" || id == "tailcc" ||
                   id == "webkit_jscc" || id == "anyregcc" ||
                   id == "cxx_fast_tlscc" || id == "swiftcc" || id == "swifttailcc" ||
                   id == "cfguard_checkcc" || id == "ghccc" || id == "hhvmcc" || id == "hhvm_ccc" ||
                   id == "intel_ocl_bicc" || id == "win64cc" {
                    self.advance();
                    // Some calling conventions have parameters: riscv_vls_cc(N)
                    if self.check(&Token::LParen) {
                        self.advance(); // consume (
                        while !self.check(&Token::RParen) && !self.is_at_end() {
                            self.advance();
                        }
                        self.match_token(&Token::RParen); // consume )
                    }
                    continue;
                }
            }

            // Check for old linkage types (3.2 era): linker_private, linker_private_weak, linker_private_weak_def_auto, linkonce_odr_auto_hide
            if let Some(Token::Identifier(id)) = self.peek() {
                if id == "linker_private" || id == "linker_private_weak" ||
                   id == "linker_private_weak_def_auto" || id == "linkonce_odr_auto_hide" {
                    self.advance();
                    continue;
                }
            }

            break;
        }

        // Handle addrspace modifier: addrspace(N) or addrspace("A")
        if self.match_token(&Token::Addrspace) {
            if self.match_token(&Token::LParen) {
                // Parse address space number or symbolic string
                if let Some(Token::Integer(_)) | Some(Token::StringLit(_)) = self.peek() {
                    self.advance();
                }
                self.match_token(&Token::RParen);
            }
        }
    }

    fn parse_return_attributes(&mut self) -> crate::function::ReturnAttributes {
        use crate::function::ReturnAttributes;
        let mut attrs = ReturnAttributes::default();

        loop {
            // Skip numbered attribute groups (#0, #1, etc.)
            if self.check(&Token::Hash) || self.check_attr_group_id() {
                self.advance();
                continue;
            }

            // Parse return attributes
            match self.peek() {
                Some(Token::Zeroext) => {
                    self.advance();
                    attrs.zeroext = true;
                    continue;
                },
                Some(Token::Signext) => {
                    self.advance();
                    attrs.signext = true;
                    continue;
                },
                Some(Token::Inreg) => {
                    self.advance();
                    attrs.inreg = true;
                    continue;
                },
                Some(Token::Noalias) => {
                    self.advance();
                    attrs.noalias = true;
                    continue;
                },
                Some(Token::Nonnull) => {
                    self.advance();
                    attrs.nonnull = true;
                    continue;
                },
                Some(Token::Align) => {
                    self.advance();
                    if let Some(Token::Integer(n)) = self.peek() {
                        attrs.align = Some(*n as u32);
                        self.advance();
                    }
                    continue;
                },
                Some(Token::Swifterror) => {
                    self.advance();
                    attrs.swifterror = true;
                    continue;
                },
                _ => {}
            }

            // Attributes with parameters: dereferenceable(N), dereferenceable_or_null(N)
            if self.match_token(&Token::Dereferenceable) {
                if self.check(&Token::LParen) {
                    self.advance();
                    if let Some(Token::Integer(n)) = self.peek() {
                        attrs.dereferenceable = Some(*n as u64);
                        self.advance();
                    }
                    self.match_token(&Token::RParen);
                }
                continue;
            }

            // Skip other attributes we don't parse yet (but immarg is tracked for validation)
            if self.match_token(&Token::Nocapture) ||
               self.match_token(&Token::Nest) ||
               self.match_token(&Token::Readonly) ||
               self.match_token(&Token::Writeonly) ||
               self.match_token(&Token::Swiftself) {
                continue;
            }

            // Track immarg in return position (will be validated as error)
            if self.match_token(&Token::Immarg) {
                attrs.has_immarg = true; // Track this for validation
                continue;
            }

            // Handle attributes with type parameters we need to skip
            if self.match_token(&Token::Byval) ||
               self.match_token(&Token::Sret) ||
               self.match_token(&Token::Inalloca) ||
               self.match_token(&Token::Preallocated) {
                if self.check(&Token::LParen) {
                    self.advance();
                    while !self.check(&Token::RParen) && !self.is_at_end() {
                        self.advance();
                    }
                    self.match_token(&Token::RParen);
                }
                continue;
            }

            // Skip dereferenceable_or_null
            if self.match_token(&Token::Dereferenceable_or_null) {
                if self.check(&Token::LParen) {
                    self.advance();
                    while !self.check(&Token::RParen) && !self.is_at_end() {
                        self.advance();
                    }
                    self.match_token(&Token::RParen);
                }
                continue;
            }

            // Handle identifier-based attributes (noundef, etc.)
            if let Some(Token::Identifier(attr)) = self.peek() {
                if matches!(attr.as_str(), "noundef") {
                    self.advance();
                    attrs.noundef = true;
                    continue;
                }
                // Handle identifier-based attributes with parameters: nofpclass(...), range(...), etc.
                if matches!(attr.as_str(), "nofpclass" | "range") {
                    self.advance();
                    if self.check(&Token::LParen) {
                        self.advance(); // consume (
                        // Skip tokens until we find )
                        while !self.check(&Token::RParen) && !self.is_at_end() {
                            self.advance();
                        }
                        self.match_token(&Token::RParen); // consume )
                    }
                    continue;
                }
            }

            break;
        }

        attrs
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
               self.match_token(&Token::Nest) ||
               self.match_token(&Token::Nonnull) ||
               self.match_token(&Token::Readonly) ||
               self.match_token(&Token::Writeonly) ||
               self.match_token(&Token::Swifterror) ||
               self.match_token(&Token::Swiftself) ||
               self.match_token(&Token::Immarg) {
                continue;
            }

            // Attributes with parameters: dereferenceable(N), dereferenceable_or_null(N)
            if self.match_token(&Token::Dereferenceable) || self.match_token(&Token::Dereferenceable_or_null) {
                if self.check(&Token::LParen) {
                    self.advance();
                    while !self.check(&Token::RParen) && !self.is_at_end() {
                        self.advance();
                    }
                    self.match_token(&Token::RParen);
                }
                continue;
            }

            // Handle attributes with optional type parameters: byval(type), sret(type), byref(type), inalloca(type), preallocated(type)
            if self.match_token(&Token::Byval) ||
               self.match_token(&Token::Sret) ||
               self.match_token(&Token::Inalloca) ||
               self.match_token(&Token::Preallocated) {
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

            // Handle identifier-based attributes (noundef, nonnull, nofpclass, range, etc.)
            if let Some(Token::Identifier(attr)) = self.peek() {
                if matches!(attr.as_str(), "noundef" | "nonnull" | "readonly" | "writeonly" |
                                          "readnone" | "returned" | "noreturn" | "nounwind" |
                                          "allocalign" | "allocsize" | "initializes" | "nofpclass" | "range") {
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

            // Handle string attributes: "name" or "name"="value"
            if matches!(self.peek(), Some(Token::StringLit(_))) {
                self.advance(); // consume string
                // Check for optional ="value" part
                if self.match_token(&Token::Equal) {
                    if matches!(self.peek(), Some(Token::StringLit(_))) {
                        self.advance(); // consume value
                    }
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

            // Handle attributes with type parameters: byval(type), sret(type), inalloca(type), preallocated(type)
            if self.match_token(&Token::Byval) ||
               self.match_token(&Token::Sret) ||
               self.match_token(&Token::Inalloca) ||
               self.match_token(&Token::Preallocated) {
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

            // Handle identifier-based attributes with type parameters: byref(type), elementtype(type), preallocated(type), range(type val, val), nofpclass(...), captures(...)
            if let Some(Token::Identifier(attr)) = self.peek() {
                if matches!(attr.as_str(), "byref" | "elementtype" | "preallocated" | "range" | "nofpclass" | "captures") {
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

    fn parse_parameter_attributes(&mut self) -> crate::function::ParameterAttributes {
        use crate::function::ParameterAttributes;
        let mut attrs = ParameterAttributes::default();
        let mut attr_count = 0;
        const MAX_ATTR_PARSE: usize = 50;

        while !self.is_at_end() && attr_count < MAX_ATTR_PARSE {
            // Stop when we hit a local identifier (parameter name), comma, rparen, or type token
            if self.check_local_ident() || self.check(&Token::Comma) ||
               self.check(&Token::RParen) || self.check_type_token() {
                break;
            }

            // Parse structured parameter attributes
            match self.peek() {
                Some(Token::Zeroext) => {
                    self.advance();
                    attrs.zeroext = true;
                    attr_count += 1;
                    continue;
                },
                Some(Token::Signext) => {
                    self.advance();
                    attrs.signext = true;
                    attr_count += 1;
                    continue;
                },
                Some(Token::Inreg) => {
                    self.advance();
                    attrs.inreg = true;
                    attr_count += 1;
                    continue;
                },
                Some(Token::Noalias) => {
                    self.advance();
                    attrs.noalias = true;
                    attr_count += 1;
                    continue;
                },
                Some(Token::Nocapture) => {
                    self.advance();
                    attrs.nocapture = true;
                    attr_count += 1;
                    continue;
                },
                Some(Token::Nest) => {
                    self.advance();
                    attrs.nest = true;
                    attr_count += 1;
                    continue;
                },
                Some(Token::Returned) => {
                    self.advance();
                    attrs.returned = true;
                    attr_count += 1;
                    continue;
                },
                Some(Token::Nonnull) => {
                    self.advance();
                    attrs.nonnull = true;
                    attr_count += 1;
                    continue;
                },
                Some(Token::Swiftself) => {
                    self.advance();
                    attrs.swiftself = true;
                    attr_count += 1;
                    continue;
                },
                Some(Token::Swifterror) => {
                    self.advance();
                    attrs.swifterror = true;
                    attr_count += 1;
                    continue;
                },
                Some(Token::Swiftasync) => {
                    self.advance();
                    attrs.swiftasync = true;
                    attr_count += 1;
                    continue;
                },
                Some(Token::Immarg) => {
                    self.advance();
                    attrs.immarg = true;
                    attr_count += 1;
                    continue;
                },
                Some(Token::Align) => {
                    self.advance();
                    if let Some(Token::Integer(n)) = self.peek() {
                        attrs.align = Some(*n as u32);
                        self.advance();
                    }
                    attr_count += 1;
                    continue;
                },
                _ => {}
            }

            // Handle attributes with type parameters: byval(type), sret(type), inalloca(type)
            if self.match_token(&Token::Byval) {
                if self.check(&Token::LParen) {
                    self.advance(); // consume (
                    if let Ok(ty) = self.parse_type() {
                        attrs.byval = Some(ty);
                    }
                    self.match_token(&Token::RParen); // consume )
                }
                attr_count += 1;
                continue;
            }

            if self.match_token(&Token::Sret) {
                if self.check(&Token::LParen) {
                    self.advance(); // consume (
                    if let Ok(ty) = self.parse_type() {
                        attrs.sret = Some(ty);
                    }
                    self.match_token(&Token::RParen); // consume )
                }
                attr_count += 1;
                continue;
            }

            if self.match_token(&Token::Inalloca) {
                if self.check(&Token::LParen) {
                    self.advance(); // consume (
                    if let Ok(ty) = self.parse_type() {
                        attrs.inalloca = Some(ty);
                    }
                    self.match_token(&Token::RParen); // consume )
                }
                attr_count += 1;
                continue;
            }

            // Handle byref(type) - identifier-based attribute
            if let Some(Token::Identifier(attr)) = self.peek() {
                if attr == "byref" {
                    self.advance();
                    if self.check(&Token::LParen) {
                        self.advance(); // consume (
                        if let Ok(ty) = self.parse_type() {
                            attrs.byref = Some(ty);
                        }
                        self.match_token(&Token::RParen); // consume )
                    }
                    attr_count += 1;
                    continue;
                }
            }

            // Handle dereferenceable(N)
            if let Some(Token::Identifier(attr)) = self.peek() {
                if attr == "dereferenceable" {
                    self.advance();
                    if self.check(&Token::LParen) {
                        self.advance(); // consume (
                        if let Some(Token::Integer(n)) = self.peek() {
                            attrs.dereferenceable = Some(*n as u64);
                            self.advance();
                        }
                        self.match_token(&Token::RParen); // consume )
                    }
                    attr_count += 1;
                    continue;
                }
            }

            // Handle dead_on_return
            if let Some(Token::Identifier(attr)) = self.peek() {
                if attr == "dead_on_return" {
                    self.advance();
                    attrs.dead_on_return = true;
                    attr_count += 1;
                    continue;
                }
            }

            // Handle dead_on_unwind
            if let Some(Token::Identifier(attr)) = self.peek() {
                if attr == "dead_on_unwind" {
                    self.advance();
                    attrs.dead_on_unwind = true;
                    attr_count += 1;
                    continue;
                }
            }

            // Handle other attributes that we need to skip but don't parse yet
            if self.match_token(&Token::Preallocated) {
                if self.check(&Token::LParen) {
                    self.advance(); // consume (
                    while !self.check(&Token::RParen) && !self.is_at_end() {
                        self.advance();
                    }
                    self.match_token(&Token::RParen); // consume )
                }
                attr_count += 1;
                continue;
            }

            // Handle identifier-based attributes with parameters
            if let Some(Token::Identifier(attr)) = self.peek() {
                if matches!(attr.as_str(), "byref" | "elementtype" | "preallocated" | "range" | "nofpclass" | "captures") {
                    self.advance();
                    if self.check(&Token::LParen) {
                        self.advance(); // consume (
                        while !self.check(&Token::RParen) && !self.is_at_end() {
                            self.advance();
                        }
                        self.match_token(&Token::RParen); // consume )
                    }
                    attr_count += 1;
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
                    attr_count += 1;
                    continue;
                }
            }

            // If we got here, skip this token and move on
            self.advance();
            attr_count += 1;
        }

        attrs
    }

    fn parse_function_attributes(&mut self) -> crate::function::FunctionAttributes {
        use crate::function::FunctionAttributes;
        let mut attrs = FunctionAttributes::default();

        // Parse function attributes
        while !self.is_at_end() && !self.check(&Token::LBrace) {
            // Handle attribute groups: #0, #1, etc.
            if let Some(Token::AttrGroupId(n)) = self.peek() {
                attrs.attribute_groups.push(format!("#{}", n));
                self.advance();
                continue;
            }

            // Parse structured function attributes
            match self.peek() {
                Some(Token::Noreturn) => { self.advance(); attrs.noreturn = true; },
                Some(Token::Noinline) => { self.advance(); attrs.noinline = true; },
                Some(Token::Alwaysinline) => { self.advance(); attrs.alwaysinline = true; },
                Some(Token::Inlinehint) => { self.advance(); attrs.inlinehint = true; },
                Some(Token::Optsize) => { self.advance(); attrs.optsize = true; },
                Some(Token::Optnone) => { self.advance(); attrs.optnone = true; },
                Some(Token::Minsize) => { self.advance(); attrs.minsize = true; },
                Some(Token::Nounwind) => { self.advance(); attrs.nounwind = true; },
                Some(Token::Norecurse) => { self.advance(); attrs.norecurse = true; },
                Some(Token::Willreturn) => { self.advance(); attrs.willreturn = true; },
                Some(Token::Nosync) => { self.advance(); attrs.nosync = true; },
                Some(Token::Readnone) => { self.advance(); attrs.readnone = true; },
                Some(Token::Readonly) => { self.advance(); attrs.readonly = true; },
                Some(Token::Writeonly) => { self.advance(); attrs.writeonly = true; },
                Some(Token::Argmemonly) => { self.advance(); attrs.argmemonly = true; },
                Some(Token::Speculatable) => { self.advance(); attrs.speculatable = true; },
                Some(Token::Returns_twice) => { self.advance(); attrs.returns_twice = true; },
                Some(Token::Ssp) => { self.advance(); attrs.ssp = true; },
                Some(Token::Sspreq) => { self.advance(); attrs.sspreq = true; },
                Some(Token::Sspstrong) => { self.advance(); attrs.sspstrong = true; },
                Some(Token::Uwtable) => { self.advance(); attrs.uwtable = true; },
                Some(Token::Cold) => { self.advance(); attrs.cold = true; },
                Some(Token::Hot) => { self.advance(); attrs.hot = true; },
                Some(Token::Naked) => { self.advance(); attrs.naked = true; },
                Some(Token::Builtin) => { self.advance(); attrs.builtin = true; },
                Some(Token::Immarg) => { self.advance(); attrs.has_immarg = true; },
                _ => {
                    // Handle other attributes and unrecognized ones
                    if self.match_token(&Token::Inaccessiblememonly) ||
                       self.match_token(&Token::Inaccessiblemem_or_argmemonly) ||
                       self.match_token(&Token::Sanitize_address) ||
                       self.match_token(&Token::Sanitize_thread) ||
                       self.match_token(&Token::Sanitize_memory) ||
                       self.match_token(&Token::Sanitize_hwaddress) ||
                       self.match_token(&Token::Safestack) ||
                       self.match_token(&Token::Nocf_check) ||
                       self.match_token(&Token::Shadowcallstack) ||
                       self.match_token(&Token::Mustprogress) ||
                       self.match_token(&Token::Strictfp) ||
                       self.match_token(&Token::Nobuiltin) ||
                       self.match_token(&Token::Noduplicate) ||
                       self.match_token(&Token::Noimplicitfloat) ||
                       self.match_token(&Token::Nomerge) ||
                       self.match_token(&Token::Nonlazybind) ||
                       self.match_token(&Token::Noredzone) ||
                       self.match_token(&Token::Null_pointer_is_valid) ||
                       self.match_token(&Token::Optforfuzzing) ||
                       self.match_token(&Token::Thunk) {
                        // These are stored but not in structured form yet
                        continue;
                    }

                    // Handle metadata
                    if self.is_metadata_token() {
                        self.skip_metadata();
                        continue;
                    }

                    // Handle attributes with parameters
                    if self.match_token(&Token::Preallocated) || self.match_token(&Token::Vscale_range) {
                        if self.check(&Token::LParen) {
                            self.advance();
                            while !self.check(&Token::RParen) && !self.is_at_end() {
                                self.advance();
                            }
                            self.match_token(&Token::RParen);
                        }
                        continue;
                    }

                    // Handle identifier-based attributes
                    if let Some(Token::Identifier(attr)) = self.peek() {
                        // Parse allockind("alloc,zeroed") or allockind("free")
                        if attr == "allockind" {
                            self.advance(); // consume 'allockind'
                            if self.check(&Token::LParen) {
                                self.advance(); // consume (
                                // Expect a string literal
                                if let Some(Token::StringLit(s)) = self.peek() {
                                    // Parse comma-separated kinds
                                    let kinds: Vec<String> = s.split(',')
                                        .map(|k| k.trim().to_string())
                                        .collect();
                                    attrs.allockind = Some(kinds);
                                    self.advance(); // consume string
                                }
                                self.match_token(&Token::RParen); // consume )
                            }
                            continue;
                        }

                        // Parse allocsize(0) or allocsize(0, 1)
                        if attr == "allocsize" {
                            self.advance(); // consume 'allocsize'
                            if self.check(&Token::LParen) {
                                self.advance(); // consume (
                                let mut indices = Vec::new();
                                // Parse first index
                                if let Some(Token::Integer(idx)) = self.peek() {
                                    indices.push(*idx as usize);
                                    self.advance();
                                }
                                // Check for second index
                                if self.match_token(&Token::Comma) {
                                    if let Some(Token::Integer(idx)) = self.peek() {
                                        indices.push(*idx as usize);
                                        self.advance();
                                    }
                                }
                                self.match_token(&Token::RParen); // consume )
                                if !indices.is_empty() {
                                    attrs.allocsize = Some(indices);
                                }
                            }
                            continue;
                        }

                        if matches!(attr.as_str(), "memory" | "convergent" | "inaccessiblememonly" |
                                                  "null_pointer_is_valid" | "optforfuzzing" | "presplitcoroutine" |
                                                  "sanitize_address_dyninit" | "allocptr" |
                                                  "alloc-family" | "fn_ret_thunk_extern") {
                            attrs.other_attributes.push(attr.clone());
                            self.advance();
                            // Some have parameters
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

                    // Handle inline string attributes: "key"="value" or "key"
                    if let Some(Token::StringLit(key)) = self.peek().cloned() {
                        self.advance();
                        if self.match_token(&Token::Equal) {
                            if let Some(Token::StringLit(value)) = self.peek().cloned() {
                                attrs.string_attributes.insert(key, value);
                                self.advance();
                            } else {
                                // No value, store empty string
                                attrs.string_attributes.insert(key, String::new());
                            }
                        } else {
                            // Key without value
                            attrs.string_attributes.insert(key, String::new());
                        }
                        continue;
                    }

                    // Exit loop if we don't recognize the token as an attribute
                    break;
                }
            }
        }

        // Note: String attributes from attribute groups are applied in a second pass
        // after all attribute groups have been parsed (see apply_attribute_groups method)

        attrs
    }

    fn skip_function_attributes(&mut self) {
        // Skip function attributes like noinline, alwaysinline, etc.
        while !self.is_at_end() && !self.check(&Token::LBrace) {
            if self.check(&Token::Hash) || self.check_attr_group_id() {
                self.advance();
                continue;
            }

            // Skip keyword-based function attributes
            if self.match_token(&Token::Noreturn) ||
               self.match_token(&Token::Noinline) ||
               self.match_token(&Token::Alwaysinline) ||
               self.match_token(&Token::Inlinehint) ||
               self.match_token(&Token::Optsize) ||
               self.match_token(&Token::Optnone) ||
               self.match_token(&Token::Minsize) ||
               self.match_token(&Token::Nounwind) ||
               self.match_token(&Token::Norecurse) ||
               self.match_token(&Token::Willreturn) ||
               self.match_token(&Token::Nosync) ||
               self.match_token(&Token::Readnone) ||
               self.match_token(&Token::Readonly) ||
               self.match_token(&Token::Writeonly) ||
               self.match_token(&Token::Argmemonly) ||
               self.match_token(&Token::Inaccessiblememonly) ||
               self.match_token(&Token::Inaccessiblemem_or_argmemonly) ||
               self.match_token(&Token::Speculatable) ||
               self.match_token(&Token::Returns_twice) ||
               self.match_token(&Token::Ssp) ||
               self.match_token(&Token::Sspreq) ||
               self.match_token(&Token::Sspstrong) ||
               self.match_token(&Token::Sanitize_address) ||
               self.match_token(&Token::Sanitize_thread) ||
               self.match_token(&Token::Sanitize_memory) ||
               self.match_token(&Token::Sanitize_hwaddress) ||
               self.match_token(&Token::Safestack) ||
               self.match_token(&Token::Uwtable) ||
               self.match_token(&Token::Nocf_check) ||
               self.match_token(&Token::Shadowcallstack) ||
               self.match_token(&Token::Mustprogress) ||
               self.match_token(&Token::Strictfp) ||
               self.match_token(&Token::Naked) ||
               self.match_token(&Token::Builtin) ||
               self.match_token(&Token::Cold) ||
               self.match_token(&Token::Hot) ||
               self.match_token(&Token::Nobuiltin) ||
               self.match_token(&Token::Noduplicate) ||
               self.match_token(&Token::Noimplicitfloat) ||
               self.match_token(&Token::Nomerge) ||
               self.match_token(&Token::Nonlazybind) ||
               self.match_token(&Token::Noredzone) ||
               self.match_token(&Token::Null_pointer_is_valid) ||
               self.match_token(&Token::Optforfuzzing) ||
               self.match_token(&Token::Thunk) {
                continue;
            }

            // Skip metadata attachments: !dbg !12
            if self.is_metadata_token() {
                self.skip_metadata();
                continue;
            }

            // Handle attributes with type parameters that can appear on calls: preallocated(type)
            if self.match_token(&Token::Preallocated) {
                if self.check(&Token::LParen) {
                    self.advance();
                    while !self.check(&Token::RParen) && !self.is_at_end() {
                        self.advance();
                    }
                    self.match_token(&Token::RParen);
                }
                continue;
            }

            // Skip identifier-based attributes (memory(...), vscale_range(...), etc.)
            if let Some(Token::Identifier(attr)) = self.peek() {
                if matches!(attr.as_str(), "memory" | "convergent" | "inaccessiblememonly" |
                                          "null_pointer_is_valid" | "optforfuzzing" | "presplitcoroutine" |
                                          "sanitize_address_dyninit" | "allockind" | "allocptr" |
                                          "alloc-family" | "fn_ret_thunk_extern") {
                    self.advance();
                    // Some have parameters: memory(read)
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

            // Handle vscale_range with parameters
            if self.match_token(&Token::Vscale_range) {
                if self.check(&Token::LParen) {
                    self.advance();
                    while !self.check(&Token::RParen) && !self.is_at_end() {
                        self.advance();
                    }
                    self.match_token(&Token::RParen);
                }
                continue;
            }

            // No more recognized attributes
            break;
        }
    }

    fn skip_instruction_flags(&mut self) {
        loop {
            // Integer arithmetic flags (keyword tokens)
            if self.match_token(&Token::Nuw) ||
               self.match_token(&Token::Nsw) ||
               self.match_token(&Token::Exact) ||
               self.match_token(&Token::Inbounds) {
                continue;
            }

            // Handle inrange flag with optional parameters: inrange or inrange(-8, 16)
            if self.match_token(&Token::Inrange) {
                if self.check(&Token::LParen) {
                    self.advance(); // consume (
                    while !self.check(&Token::RParen) && !self.is_at_end() {
                        self.advance(); // skip all tokens inside
                    }
                    self.match_token(&Token::RParen); // consume )
                }
                continue;
            }

            // Fast-math flags and other identifier-based flags (identifiers)
            if let Some(Token::Identifier(id)) = self.peek() {
                if matches!(id.as_str(), "fast" | "nnan" | "ninf" | "nsz" | "arcp" |
                                         "contract" | "afn" | "reassoc" | "nneg" | "disjoint" | "samesign" | "nusw") {
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

    fn parse_load_store_attributes(&mut self) -> Option<u64> {
        let mut alignment: Option<u64> = None;
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
                if let Some(Token::Integer(val)) = self.peek() {
                    alignment = Some(*val as u64);
                    self.advance();
                }
            } else if self.match_token(&Token::Volatile) {
                // consumed
            } else {
                self.current -= 1; // put back the comma
                break;
            }
        }
        alignment
    }

    fn skip_load_store_attributes(&mut self) {
        self.parse_load_store_attributes();
    }

    fn skip_atomic_ordering(&mut self) -> bool {
        // Atomic orderings are the same as memory orderings
        self.skip_memory_ordering()
    }

    fn skip_to_matching_paren(&mut self) {
        // Skip all tokens until we find the matching closing parenthesis
        let mut depth = 1;
        while depth > 0 && !self.is_at_end() {
            if self.check(&Token::LParen) {
                depth += 1;
            } else if self.check(&Token::RParen) {
                depth -= 1;
            }
            if depth > 0 {
                self.advance();
            }
        }
    }

    fn skip_to_matching_bracket(&mut self) {
        // Skip all tokens until we find the matching closing bracket
        let mut depth = 1;
        while depth > 0 && !self.is_at_end() {
            if self.check(&Token::LBracket) {
                depth += 1;
            } else if self.check(&Token::RBracket) {
                depth -= 1;
            }
            if depth > 0 {
                self.advance();
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
    let module = parser.parse_module(source)?;

    // Verify the module after parsing
    match crate::verification::verify_module(&module) {
        Ok(_) => Ok(module),
        Err(errors) => {
            // Convert verification errors to parse error
            let error_msg = errors.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            Err(ParseError::InvalidSyntax {
                message: format!("Verification failed: {}", error_msg),
                position: 0,
            })
        }
    }
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
