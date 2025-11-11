//! Runtime Entry Point and Support (Level 8)
//!
//! Provides the _start entry point and runtime initialization for executables.

/// Generate the _start entry point for an executable
pub fn generate_start_entry(main_function: &str) -> String {
    format!(r#"
    .section .text
    .globl _start
    .type _start, @function
_start:
    # Initialize the program
    # Clear the frame pointer
    xor %rbp, %rbp

    # Pop argc from stack
    pop %rdi

    # argv is now at rsp
    mov %rsp, %rsi

    # Align stack to 16-byte boundary (System V ABI requirement)
    and $-16, %rsp

    # Call main function
    call {}

    # Exit with main's return value
    mov %rax, %rdi
    mov $60, %rax      # syscall number for exit
    syscall

    .size _start, . - _start

    # Add section for dynamic linker
    .section .interp,"a",@progbits
    .string "/lib64/ld-linux-x86-64.so.2"
"#, main_function)
}

/// Generate minimal C runtime initialization
pub fn generate_crt_init() -> String {
    r#"
    .section .init
    .globl _init
    .type _init, @function
_init:
    push %rbp
    mov %rsp, %rbp
    # Initialize constructors here if needed
    pop %rbp
    ret

    .section .fini
    .globl _fini
    .type _fini, @function
_fini:
    push %rbp
    mov %rsp, %rbp
    # Call destructors here if needed
    pop %rbp
    ret
"#.to_string()
}

/// Generate program headers for dynamic linking
pub fn generate_program_headers() -> Vec<ProgramHeader> {
    vec![
        ProgramHeader {
            p_type: PT_LOAD,    // Loadable segment
            p_flags: PF_R | PF_X, // Read + Execute
            p_offset: 0,
            p_vaddr: 0x400000,
            p_paddr: 0x400000,
            p_filesz: 0,
            p_memsz: 0,
            p_align: 0x1000,
        },
        ProgramHeader {
            p_type: PT_LOAD,    // Loadable segment
            p_flags: PF_R | PF_W, // Read + Write
            p_offset: 0,
            p_vaddr: 0x600000,
            p_paddr: 0x600000,
            p_filesz: 0,
            p_memsz: 0,
            p_align: 0x1000,
        },
        ProgramHeader {
            p_type: PT_DYNAMIC, // Dynamic linking info
            p_flags: PF_R | PF_W,
            p_offset: 0,
            p_vaddr: 0,
            p_paddr: 0,
            p_filesz: 0,
            p_memsz: 0,
            p_align: 8,
        },
    ]
}

/// ELF program header
#[derive(Debug, Clone)]
pub struct ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

// Program header types
pub const PT_LOAD: u32 = 1;
pub const PT_DYNAMIC: u32 = 2;
pub const PT_INTERP: u32 = 3;

// Program header flags
pub const PF_X: u32 = 1;  // Executable
pub const PF_W: u32 = 2;  // Writable
pub const PF_R: u32 = 4;  // Readable

impl ProgramHeader {
    /// Convert to bytes (64-bit ELF format)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.p_type.to_le_bytes());
        bytes.extend_from_slice(&self.p_flags.to_le_bytes());
        bytes.extend_from_slice(&self.p_offset.to_le_bytes());
        bytes.extend_from_slice(&self.p_vaddr.to_le_bytes());
        bytes.extend_from_slice(&self.p_paddr.to_le_bytes());
        bytes.extend_from_slice(&self.p_filesz.to_le_bytes());
        bytes.extend_from_slice(&self.p_memsz.to_le_bytes());
        bytes.extend_from_slice(&self.p_align.to_le_bytes());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_start_entry() {
        let entry = generate_start_entry("main");
        assert!(entry.contains("_start"));
        assert!(entry.contains("call main"));
        assert!(entry.contains("syscall"));
    }

    #[test]
    fn test_generate_crt_init() {
        let crt = generate_crt_init();
        assert!(crt.contains("_init"));
        assert!(crt.contains("_fini"));
    }

    #[test]
    fn test_program_headers() {
        let headers = generate_program_headers();
        assert!(!headers.is_empty());
        assert_eq!(headers[0].p_type, PT_LOAD);
    }

    #[test]
    fn test_program_header_bytes() {
        let header = ProgramHeader {
            p_type: PT_LOAD,
            p_flags: PF_R | PF_X,
            p_offset: 0,
            p_vaddr: 0x400000,
            p_paddr: 0x400000,
            p_filesz: 0x1000,
            p_memsz: 0x1000,
            p_align: 0x1000,
        };
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), 56); // Size of 64-bit program header
    }
}
