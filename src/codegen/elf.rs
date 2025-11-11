//! ELF Object File Generation
//!
//! This module implements ELF (Executable and Linkable Format) object file generation
//! for x86-64 Linux systems.

use std::collections::HashMap;

/// ELF file header (64-bit)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Elf64Header {
    pub e_ident: [u8; 16],      // ELF identification
    pub e_type: u16,            // Object file type
    pub e_machine: u16,         // Architecture
    pub e_version: u32,         // Object file version
    pub e_entry: u64,           // Entry point virtual address
    pub e_phoff: u64,           // Program header table file offset
    pub e_shoff: u64,           // Section header table file offset
    pub e_flags: u32,           // Processor-specific flags
    pub e_ehsize: u16,          // ELF header size in bytes
    pub e_phentsize: u16,       // Program header table entry size
    pub e_phnum: u16,           // Program header table entry count
    pub e_shentsize: u16,       // Section header table entry size
    pub e_shnum: u16,           // Section header table entry count
    pub e_shstrndx: u16,        // Section header string table index
}

/// ELF section header (64-bit)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Elf64SectionHeader {
    pub sh_name: u32,           // Section name (string table index)
    pub sh_type: u32,           // Section type
    pub sh_flags: u64,          // Section flags
    pub sh_addr: u64,           // Section virtual addr at execution
    pub sh_offset: u64,         // Section file offset
    pub sh_size: u64,           // Section size in bytes
    pub sh_link: u32,           // Link to another section
    pub sh_info: u32,           // Additional section information
    pub sh_addralign: u64,      // Section alignment
    pub sh_entsize: u64,        // Entry size if section holds table
}

/// ELF symbol table entry (64-bit)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Elf64Symbol {
    pub st_name: u32,           // Symbol name (string table index)
    pub st_info: u8,            // Symbol type and binding
    pub st_other: u8,           // Symbol visibility
    pub st_shndx: u16,          // Section index
    pub st_value: u64,          // Symbol value
    pub st_size: u64,           // Symbol size
}

/// ELF relocation entry (64-bit)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Elf64Rela {
    pub r_offset: u64,          // Address
    pub r_info: u64,            // Relocation type and symbol index
    pub r_addend: i64,          // Addend
}

/// ELF object file builder
pub struct ElfObjectFile {
    /// Machine code sections
    sections: Vec<Section>,
    /// Symbol table
    symbols: Vec<Symbol>,
    /// Relocations
    relocations: Vec<Relocation>,
    /// String table
    strings: Vec<String>,
}

/// A section in the object file
#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub typ: SectionType,
    pub data: Vec<u8>,
    pub alignment: u64,
}

/// Section types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    Text,       // Executable code
    Data,       // Initialized data
    Bss,        // Uninitialized data
    Rodata,     // Read-only data
    SymTab,     // Symbol table
    StrTab,     // String table
    RelA,       // Relocations with addends
}

/// Symbol in the symbol table
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub value: u64,
    pub size: u64,
    pub binding: SymbolBinding,
    pub typ: SymbolType,
    pub section_index: u16,
}

/// Symbol binding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolBinding {
    Local,
    Global,
    Weak,
}

/// Symbol type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    NoType,
    Object,
    Function,
    Section,
    File,
}

/// Relocation entry
#[derive(Debug, Clone)]
pub struct Relocation {
    pub offset: u64,
    pub symbol: String,
    pub typ: RelocationType,
    pub addend: i64,
}

/// Relocation types (x86-64)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelocationType {
    /// Direct 64-bit relocation
    R_X86_64_64,
    /// PC-relative 32-bit
    R_X86_64_PC32,
    /// 32-bit GOT entry
    R_X86_64_GOT32,
    /// 32-bit PLT address
    R_X86_64_PLT32,
}

impl ElfObjectFile {
    /// Create a new ELF object file
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            symbols: Vec::new(),
            relocations: Vec::new(),
            strings: Vec::new(),
        }
    }

    /// Add a section
    pub fn add_section(&mut self, section: Section) {
        self.sections.push(section);
    }

    /// Add a symbol
    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.push(symbol);
    }

    /// Add a relocation
    pub fn add_relocation(&mut self, relocation: Relocation) {
        self.relocations.push(relocation);
    }

    /// Generate the ELF file as bytes
    pub fn generate(&self) -> Vec<u8> {
        let mut output = Vec::new();

        // Write ELF header
        let header = self.create_elf_header();
        output.extend_from_slice(&self.header_to_bytes(&header));

        // Write section data
        for section in &self.sections {
            output.extend_from_slice(&section.data);
        }

        // Write section headers
        for section in &self.sections {
            let sh = self.create_section_header(section);
            output.extend_from_slice(&self.section_header_to_bytes(&sh));
        }

        output
    }

    fn create_elf_header(&self) -> Elf64Header {
        Elf64Header {
            e_ident: [
                0x7f, b'E', b'L', b'F',  // Magic number
                2,                        // 64-bit
                1,                        // Little endian
                1,                        // ELF version
                0,                        // System V ABI
                0, 0, 0, 0, 0, 0, 0, 0,  // Padding
            ],
            e_type: 1,                   // ET_REL (relocatable file)
            e_machine: 62,               // x86-64
            e_version: 1,
            e_entry: 0,
            e_phoff: 0,
            e_shoff: 64,                 // Section header offset
            e_flags: 0,
            e_ehsize: 64,                // ELF header size
            e_phentsize: 0,
            e_phnum: 0,
            e_shentsize: 64,             // Section header size
            e_shnum: self.sections.len() as u16,
            e_shstrndx: 0,
        }
    }

    fn create_section_header(&self, section: &Section) -> Elf64SectionHeader {
        Elf64SectionHeader {
            sh_name: 0,
            sh_type: match section.typ {
                SectionType::Text => 1,    // SHT_PROGBITS
                SectionType::Data => 1,
                SectionType::Bss => 8,     // SHT_NOBITS
                SectionType::Rodata => 1,
                SectionType::SymTab => 2,  // SHT_SYMTAB
                SectionType::StrTab => 3,  // SHT_STRTAB
                SectionType::RelA => 4,    // SHT_RELA
            },
            sh_flags: match section.typ {
                SectionType::Text => 6,    // SHF_ALLOC | SHF_EXECINSTR
                SectionType::Data => 3,    // SHF_WRITE | SHF_ALLOC
                _ => 0,
            },
            sh_addr: 0,
            sh_offset: 64,
            sh_size: section.data.len() as u64,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: section.alignment,
            sh_entsize: 0,
        }
    }

    fn header_to_bytes(&self, header: &Elf64Header) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&header.e_ident);
        bytes.extend_from_slice(&header.e_type.to_le_bytes());
        bytes.extend_from_slice(&header.e_machine.to_le_bytes());
        bytes.extend_from_slice(&header.e_version.to_le_bytes());
        bytes.extend_from_slice(&header.e_entry.to_le_bytes());
        bytes.extend_from_slice(&header.e_phoff.to_le_bytes());
        bytes.extend_from_slice(&header.e_shoff.to_le_bytes());
        bytes.extend_from_slice(&header.e_flags.to_le_bytes());
        bytes.extend_from_slice(&header.e_ehsize.to_le_bytes());
        bytes.extend_from_slice(&header.e_phentsize.to_le_bytes());
        bytes.extend_from_slice(&header.e_phnum.to_le_bytes());
        bytes.extend_from_slice(&header.e_shentsize.to_le_bytes());
        bytes.extend_from_slice(&header.e_shnum.to_le_bytes());
        bytes.extend_from_slice(&header.e_shstrndx.to_le_bytes());
        bytes
    }

    fn section_header_to_bytes(&self, sh: &Elf64SectionHeader) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&sh.sh_name.to_le_bytes());
        bytes.extend_from_slice(&sh.sh_type.to_le_bytes());
        bytes.extend_from_slice(&sh.sh_flags.to_le_bytes());
        bytes.extend_from_slice(&sh.sh_addr.to_le_bytes());
        bytes.extend_from_slice(&sh.sh_offset.to_le_bytes());
        bytes.extend_from_slice(&sh.sh_size.to_le_bytes());
        bytes.extend_from_slice(&sh.sh_link.to_le_bytes());
        bytes.extend_from_slice(&sh.sh_info.to_le_bytes());
        bytes.extend_from_slice(&sh.sh_addralign.to_le_bytes());
        bytes.extend_from_slice(&sh.sh_entsize.to_le_bytes());
        bytes
    }
}

/// Linker for combining object files
pub struct Linker {
    objects: Vec<ElfObjectFile>,
    symbol_table: HashMap<String, u64>,
    /// Entry point symbol name
    entry_point: String,
}

impl Linker {
    /// Create a new linker
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            symbol_table: HashMap::new(),
            entry_point: "_start".to_string(),
        }
    }

    /// Set the entry point symbol
    pub fn set_entry_point(&mut self, symbol: String) {
        self.entry_point = symbol;
    }

    /// Add an object file to link
    pub fn add_object(&mut self, object: ElfObjectFile) {
        self.objects.push(object);
    }

    /// Build global symbol table
    fn build_symbol_table(&mut self) -> Result<(), String> {
        self.symbol_table.clear();
        let mut current_offset = 0u64;

        // First pass: collect all global symbols
        for obj in &self.objects {
            for symbol in &obj.symbols {
                if matches!(symbol.binding, SymbolBinding::Global) {
                    if self.symbol_table.contains_key(&symbol.name) {
                        return Err(format!("Duplicate symbol: {}", symbol.name));
                    }
                    self.symbol_table.insert(symbol.name.clone(), current_offset + symbol.value);
                }
            }

            // Update offset for next object
            for section in &obj.sections {
                if section.typ == SectionType::Text {
                    current_offset += section.data.len() as u64;
                }
            }
        }

        Ok(())
    }

    /// Resolve relocations
    fn resolve_relocations(&mut self, text_data: &mut Vec<u8>) -> Result<(), String> {
        let mut offset = 0;

        for obj in &self.objects {
            for reloc in &obj.relocations {
                let symbol_addr = self.symbol_table.get(&reloc.symbol)
                    .ok_or_else(|| format!("Undefined symbol: {}", reloc.symbol))?;

                let reloc_offset = offset + reloc.offset as usize;

                // Apply relocation based on type
                match reloc.typ {
                    RelocationType::R_X86_64_64 => {
                        // Direct 64-bit relocation
                        if reloc_offset + 8 <= text_data.len() {
                            let value = (*symbol_addr as i64 + reloc.addend) as u64;
                            text_data[reloc_offset..reloc_offset + 8]
                                .copy_from_slice(&value.to_le_bytes());
                        }
                    }
                    RelocationType::R_X86_64_PC32 => {
                        // PC-relative 32-bit relocation
                        if reloc_offset + 4 <= text_data.len() {
                            let pc = reloc_offset as i64;
                            let target = *symbol_addr as i64 + reloc.addend;
                            let value = (target - pc) as i32;
                            text_data[reloc_offset..reloc_offset + 4]
                                .copy_from_slice(&value.to_le_bytes());
                        }
                    }
                    _ => {
                        // Other relocation types not yet implemented
                    }
                }
            }

            // Update offset for next object
            for section in &obj.sections {
                if section.typ == SectionType::Text {
                    offset += section.data.len();
                }
            }
        }

        Ok(())
    }

    /// Link all object files into an executable
    pub fn link(&mut self) -> Result<Vec<u8>, String> {
        // Build symbol table
        self.build_symbol_table()?;

        // Collect all sections
        let mut text_data = Vec::new();
        let mut data_sections = Vec::new();

        for obj in &self.objects {
            for section in &obj.sections {
                match section.typ {
                    SectionType::Text => {
                        text_data.extend_from_slice(&section.data);
                    }
                    SectionType::Data | SectionType::Rodata => {
                        data_sections.push(section.clone());
                    }
                    _ => {}
                }
            }
        }

        // Resolve relocations
        self.resolve_relocations(&mut text_data)?;

        // Create output object file
        let mut output = ElfObjectFile::new();

        // Add text section
        if !text_data.is_empty() {
            output.add_section(Section {
                name: ".text".to_string(),
                typ: SectionType::Text,
                data: text_data,
                alignment: 16,
            });
        }

        // Add data sections
        for section in data_sections {
            output.add_section(section);
        }

        // Add symbols
        for (name, addr) in &self.symbol_table {
            output.add_symbol(Symbol {
                name: name.clone(),
                value: *addr,
                size: 0,
                binding: SymbolBinding::Global,
                typ: SymbolType::Function,
                section_index: 1,
            });
        }

        // Generate executable
        Ok(output.generate())
    }

    /// Get the entry point address
    pub fn entry_point_address(&self) -> Option<u64> {
        self.symbol_table.get(&self.entry_point).copied()
    }
}
