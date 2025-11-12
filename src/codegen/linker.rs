///! System Linker Integration
///!
///! Provides integration with system linker (ld) to create executables

use std::process::Command;
use std::path::Path;
use std::fs;
use super::CodegenError;

/// System linker wrapper
pub struct SystemLinker {
    /// Linker executable (typically "ld" or "gcc")
    linker_path: String,
    /// Use gcc as linker driver (recommended for automatic CRT linking)
    use_gcc: bool,
}

impl SystemLinker {
    /// Create a new system linker
    pub fn new() -> Self {
        Self {
            linker_path: "gcc".to_string(),
            use_gcc: true,
        }
    }

    /// Create linker using ld directly
    pub fn new_with_ld() -> Self {
        Self {
            linker_path: "ld".to_string(),
            use_gcc: false,
        }
    }

    /// Link object files into an executable
    pub fn link_executable(
        &self,
        object_files: &[&Path],
        output_path: &Path,
        link_libc: bool,
    ) -> Result<(), CodegenError> {
        if self.use_gcc {
            self.link_with_gcc(object_files, output_path, link_libc)
        } else {
            self.link_with_ld(object_files, output_path, link_libc)
        }
    }

    /// Link using gcc (recommended - handles CRT and libc automatically)
    fn link_with_gcc(
        &self,
        object_files: &[&Path],
        output_path: &Path,
        _link_libc: bool, // gcc links libc by default
    ) -> Result<(), CodegenError> {
        let mut cmd = Command::new(&self.linker_path);

        // Add object files
        for obj in object_files {
            cmd.arg(obj);
        }

        // Output file
        cmd.arg("-o");
        cmd.arg(output_path);

        // Execute linker
        let output = cmd.output().map_err(|e| {
            CodegenError::General(format!("Failed to execute gcc: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CodegenError::General(format!("Linking failed: {}", stderr)));
        }

        Ok(())
    }

    /// Link using ld directly (more control, but requires manual CRT setup)
    fn link_with_ld(
        &self,
        object_files: &[&Path],
        output_path: &Path,
        link_libc: bool,
    ) -> Result<(), CodegenError> {
        let mut cmd = Command::new(&self.linker_path);

        // Add CRT start files
        // These are needed to properly initialize the C runtime
        cmd.arg("/usr/lib/x86_64-linux-gnu/crt1.o");
        cmd.arg("/usr/lib/x86_64-linux-gnu/crti.o");

        // GCC crtbegin
        let crtbegin = self.find_crtbegin()?;
        cmd.arg(&crtbegin);

        // Add object files
        for obj in object_files {
            cmd.arg(obj);
        }

        // Add standard libraries if requested
        if link_libc {
            cmd.arg("-lc");
        }

        // Add library search paths
        cmd.arg("-L/usr/lib/x86_64-linux-gnu");
        cmd.arg("-L/lib/x86_64-linux-gnu");

        // GCC crtend
        let crtend = self.find_crtend()?;
        cmd.arg(&crtend);

        // CRT end file
        cmd.arg("/usr/lib/x86_64-linux-gnu/crtn.o");

        // Dynamic linker
        cmd.arg("--dynamic-linker");
        cmd.arg("/lib64/ld-linux-x86-64.so.2");

        // Output file
        cmd.arg("-o");
        cmd.arg(output_path);

        // Execute linker
        let output = cmd.output().map_err(|e| {
            CodegenError::General(format!("Failed to execute ld: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CodegenError::General(format!("Linking failed: {}", stderr)));
        }

        Ok(())
    }

    /// Find crtbegin.o
    fn find_crtbegin(&self) -> Result<String, CodegenError> {
        // Try common locations
        let candidates = [
            "/usr/lib/gcc/x86_64-linux-gnu/9/crtbegin.o",
            "/usr/lib/gcc/x86_64-linux-gnu/11/crtbegin.o",
            "/usr/lib/gcc/x86_64-linux-gnu/12/crtbegin.o",
            "/usr/lib/gcc/x86_64-linux-gnu/13/crtbegin.o",
        ];

        for path in &candidates {
            if Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        Err(CodegenError::General("Could not find crtbegin.o".to_string()))
    }

    /// Find crtend.o
    fn find_crtend(&self) -> Result<String, CodegenError> {
        // Try common locations
        let candidates = [
            "/usr/lib/gcc/x86_64-linux-gnu/9/crtend.o",
            "/usr/lib/gcc/x86_64-linux-gnu/11/crtend.o",
            "/usr/lib/gcc/x86_64-linux-gnu/12/crtend.o",
            "/usr/lib/gcc/x86_64-linux-gnu/13/crtend.o",
        ];

        for path in &candidates {
            if Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        Err(CodegenError::General("Could not find crtend.o".to_string()))
    }

    /// Write ELF bytes to disk as an object file
    pub fn write_object_file(path: &Path, elf_bytes: &[u8]) -> Result<(), CodegenError> {
        fs::write(path, elf_bytes).map_err(|e| {
            CodegenError::General(format!("Failed to write object file: {}", e))
        })
    }

    /// Validate an ELF file using readelf
    pub fn validate_elf(path: &Path) -> Result<String, CodegenError> {
        let output = Command::new("readelf")
            .arg("-h") // Header
            .arg(path)
            .output()
            .map_err(|e| CodegenError::General(format!("Failed to run readelf: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CodegenError::General(format!("readelf failed: {}", stderr)));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Get detailed ELF information using readelf
    pub fn get_elf_info(path: &Path) -> Result<String, CodegenError> {
        let output = Command::new("readelf")
            .arg("-a") // All information
            .arg(path)
            .output()
            .map_err(|e| CodegenError::General(format!("Failed to run readelf: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CodegenError::General(format!("readelf failed: {}", stderr)));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Disassemble an object file using objdump
    pub fn disassemble(path: &Path) -> Result<String, CodegenError> {
        let output = Command::new("objdump")
            .arg("-d") // Disassemble
            .arg(path)
            .output()
            .map_err(|e| CodegenError::General(format!("Failed to run objdump: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CodegenError::General(format!("objdump failed: {}", stderr)));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl Default for SystemLinker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linker_creation() {
        let _linker = SystemLinker::new();
        let _ld_linker = SystemLinker::new_with_ld();
    }

    #[test]
    fn test_crt_files_exist() {
        let linker = SystemLinker::new_with_ld();
        // These might not exist in all environments, so we just test the search
        let _ = linker.find_crtbegin();
        let _ = linker.find_crtend();
    }
}
