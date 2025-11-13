//! LLVM Functions
//!
//! Functions are the top-level callable entities in LLVM IR.
//! They consist of a signature (return type and parameters) and
//! a body (basic blocks).

use std::sync::{Arc, RwLock};
use std::fmt;
use crate::types::Type;
use crate::basic_block::BasicBlock;
use crate::value::Value;

/// Calling convention
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallingConvention {
    C,                         // ccc (default)
    Fast,                      // fastcc
    Cold,                      // coldcc
    Webkit_JS,                 // webkit_jscc
    AnyReg,                    // anyregcc
    PreserveMost,              // preserve_mostcc
    PreserveAll,               // preserve_allcc
    CXX_FastTLS,               // cxx_fast_tlscc
    Tail,                      // tailcc
    SwiftTail,                 // swifttailcc
    Swift,                     // swiftcc
    CFunc,                     // cfguard_checkcc
    X86_StdCall,               // x86_stdcallcc
    X86_FastCall,              // x86_fastcallcc
    X86_ThisCall,              // x86_thiscallcc
    X86_VectorCall,            // x86_vectorcallcc
    X86_RegCall,               // x86_regcallcc
    ARM_APCS,                  // arm_apcscc
    ARM_AAPCS,                 // arm_aapcscc
    ARM_AAPCS_VFP,             // arm_aapcs_vfpcc
    AArch64_VectorCall,        // aarch64_vector_pcs
    AArch64_SVE_VectorCall,    // aarch64_sve_vector_pcs
    AMDGPU_Kernel,             // amdgpu_kernel
    AMDGPU_VS,                 // amdgpu_vs
    AMDGPU_GS,                 // amdgpu_gs
    AMDGPU_PS,                 // amdgpu_ps
    AMDGPU_CS,                 // amdgpu_cs
    AMDGPU_HS,                 // amdgpu_hs
    AMDGPU_LS,                 // amdgpu_ls
    AMDGPU_ES,                 // amdgpu_es
    AMDGPU_CS_Chain,           // amdgpu_cs_chain
    AMDGPU_CS_Chain_Preserve,  // amdgpu_cs_chain_preserve
    AMDGPU_GFX_Whole_Wave,     // amdgpu_gfx_whole_wave
    SPIR_Kernel,               // spir_kernel
    SPIR_Func,                 // spir_func
    Intel_OCL_BI,              // intel_ocl_bicc
    PTX_Kernel,                // ptx_kernel
    PTX_Device,                // ptx_device
}

impl Default for CallingConvention {
    fn default() -> Self {
        CallingConvention::C
    }
}

/// Function attributes
#[derive(Debug, Clone, Default)]
pub struct FunctionAttributes {
    // Function-level attributes
    pub noinline: bool,
    pub alwaysinline: bool,
    pub inlinehint: bool,
    pub optsize: bool,
    pub optnone: bool,
    pub minsize: bool,
    pub noreturn: bool,
    pub nounwind: bool,
    pub norecurse: bool,
    pub willreturn: bool,
    pub nosync: bool,
    pub readnone: bool,
    pub readonly: bool,
    pub writeonly: bool,
    pub argmemonly: bool,
    pub speculatable: bool,
    pub returns_twice: bool,
    pub ssp: bool,
    pub sspreq: bool,
    pub sspstrong: bool,
    pub uwtable: bool,
    pub cold: bool,
    pub hot: bool,
    pub naked: bool,
    pub builtin: bool,

    // Return attributes
    pub return_attributes: ReturnAttributes,

    // Parameter attributes (indexed by parameter position)
    pub parameter_attributes: Vec<ParameterAttributes>,

    // Attribute group references (#0, #1, etc.)
    pub attribute_groups: Vec<String>,

    // Complex string attributes (allockind, allocsize, etc.)
    pub allockind: Option<Vec<String>>,  // e.g., ["alloc", "zeroed"]
    pub allocsize: Option<Vec<usize>>,   // e.g., [0] or [0, 1]

    // Other attributes as strings
    pub other_attributes: Vec<String>,
}

/// Return value attributes
#[derive(Debug, Clone, Default)]
pub struct ReturnAttributes {
    pub zeroext: bool,
    pub signext: bool,
    pub inreg: bool,
    pub noalias: bool,
    pub nonnull: bool,
    pub dereferenceable: Option<u64>,
    pub align: Option<u32>,
}

/// Parameter attributes
#[derive(Debug, Clone, Default)]
pub struct ParameterAttributes {
    pub zeroext: bool,
    pub signext: bool,
    pub inreg: bool,
    pub byval: Option<Type>,
    pub inalloca: Option<Type>,
    pub sret: Option<Type>,
    pub noalias: bool,
    pub nocapture: bool,
    pub nest: bool,
    pub returned: bool,
    pub nonnull: bool,
    pub dereferenceable: Option<u64>,
    pub swiftself: bool,
    pub swifterror: bool,
    pub swiftasync: bool,
    pub immarg: bool,
    pub align: Option<u32>,
}

/// A function in LLVM IR
#[derive(Clone)]
pub struct Function {
    data: Arc<RwLock<FunctionData>>,
}

struct FunctionData {
    name: String,
    ty: Type,
    basic_blocks: Vec<BasicBlock>,
    arguments: Vec<Value>,
    attributes: FunctionAttributes,
    calling_convention: CallingConvention,
    linkage: crate::module::Linkage,
    visibility: crate::module::Visibility,
    dll_storage_class: crate::module::DLLStorageClass,
    personality: Option<Value>,
}

impl Function {
    /// Create a new function
    pub fn new(name: String, ty: Type) -> Self {
        assert!(ty.is_function(), "Function must have function type");

        Self {
            data: Arc::new(RwLock::new(FunctionData {
                name,
                ty,
                basic_blocks: Vec::new(),
                arguments: Vec::new(),
                attributes: FunctionAttributes::default(),
                calling_convention: CallingConvention::default(),
                linkage: crate::module::Linkage::External,
                visibility: crate::module::Visibility::Default,
                dll_storage_class: crate::module::DLLStorageClass::Default,
                personality: None,
            })),
        }
    }

    /// Get the calling convention
    pub fn calling_convention(&self) -> CallingConvention {
        self.data.read().unwrap().calling_convention
    }

    /// Set the calling convention
    pub fn set_calling_convention(&self, cc: CallingConvention) {
        self.data.write().unwrap().calling_convention = cc;
    }

    /// Get the function attributes
    pub fn attributes(&self) -> FunctionAttributes {
        self.data.read().unwrap().attributes.clone()
    }

    /// Set the function attributes
    pub fn set_attributes(&self, attributes: FunctionAttributes) {
        self.data.write().unwrap().attributes = attributes;
    }

    /// Get the function linkage
    pub fn linkage(&self) -> crate::module::Linkage {
        self.data.read().unwrap().linkage
    }

    /// Set the function linkage
    pub fn set_linkage(&self, linkage: crate::module::Linkage) {
        self.data.write().unwrap().linkage = linkage;
    }

    /// Get the function visibility
    pub fn visibility(&self) -> crate::module::Visibility {
        self.data.read().unwrap().visibility
    }

    /// Set the function visibility
    pub fn set_visibility(&self, visibility: crate::module::Visibility) {
        self.data.write().unwrap().visibility = visibility;
    }

    /// Get the function DLL storage class
    pub fn dll_storage_class(&self) -> crate::module::DLLStorageClass {
        self.data.read().unwrap().dll_storage_class
    }

    /// Set the function DLL storage class
    pub fn set_dll_storage_class(&self, dll_storage_class: crate::module::DLLStorageClass) {
        self.data.write().unwrap().dll_storage_class = dll_storage_class;
    }

    /// Get the personality function
    pub fn personality(&self) -> Option<Value> {
        self.data.read().unwrap().personality.clone()
    }

    /// Set the personality function
    pub fn set_personality(&self, personality: Option<Value>) {
        self.data.write().unwrap().personality = personality;
    }

    /// Get the name of this function
    pub fn name(&self) -> String {
        self.data.read().unwrap().name.clone()
    }

    /// Get the type of this function
    pub fn get_type(&self) -> Type {
        self.data.read().unwrap().ty.clone()
    }

    /// Add a basic block to this function
    pub fn add_basic_block(&self, bb: BasicBlock) {
        let mut data = self.data.write().unwrap();
        data.basic_blocks.push(bb);
    }

    /// Get the basic blocks in this function
    pub fn basic_blocks(&self) -> Vec<BasicBlock> {
        self.data.read().unwrap().basic_blocks.clone()
    }

    /// Get the entry basic block
    pub fn entry_block(&self) -> Option<BasicBlock> {
        self.data.read().unwrap().basic_blocks.first().cloned()
    }

    /// Get the number of basic blocks in this function
    pub fn basic_block_count(&self) -> usize {
        self.data.read().unwrap().basic_blocks.len()
    }

    /// Set the function arguments
    pub fn set_arguments(&self, arguments: Vec<Value>) {
        let mut data = self.data.write().unwrap();
        data.arguments = arguments;
    }

    /// Get the function arguments
    pub fn arguments(&self) -> Vec<Value> {
        self.data.read().unwrap().arguments.clone()
    }

    /// Get a specific argument by index
    pub fn argument(&self, index: usize) -> Option<Value> {
        self.data.read().unwrap().arguments.get(index).cloned()
    }

    /// Check if this function has a body
    pub fn has_body(&self) -> bool {
        !self.data.read().unwrap().basic_blocks.is_empty()
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self.data.read().unwrap();

        write!(f, "define {} @{}(", data.ty, data.name)?;

        for (i, arg) in data.arguments.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{} {}", arg.get_type(), arg)?;
        }

        writeln!(f, ") {{")?;

        for bb in &data.basic_blocks {
            write!(f, "{}", bb)?;
        }

        writeln!(f, "}}")
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self.data.read().unwrap();
        write!(f, "Function(@{}, {} basic blocks)", data.name, data.basic_blocks.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;

    #[test]
    fn test_function_creation() {
        let ctx = Context::new();
        let i32_type = ctx.int32_type();
        let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone(), i32_type.clone()], false);

        let func = Function::new("add".to_string(), fn_type);
        assert_eq!(func.name(), "add");
        assert!(!func.has_body());
    }

    #[test]
    fn test_add_basic_block() {
        let ctx = Context::new();
        let i32_type = ctx.int32_type();
        let fn_type = ctx.function_type(i32_type, vec![], false);

        let func = Function::new("test".to_string(), fn_type);
        let bb = BasicBlock::new(Some("entry".to_string()));

        func.add_basic_block(bb);
        assert_eq!(func.basic_block_count(), 1);
        assert!(func.has_body());
    }
}
