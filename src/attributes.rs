//! LLVM Attributes System
//!
//! Attributes provide additional information about functions, parameters,
//! and return values. They can affect code generation, optimization, and
//! calling conventions.

use std::collections::HashSet;
use std::fmt;

/// Function attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctionAttribute {
    // Function-only attributes
    AlwaysInline,
    Cold,
    Hot,
    InlineHint,
    MinSize,
    Naked,
    NoBuiltin,
    NoDuplicate,
    NoImplicitFloat,
    NoInline,
    NonLazyBind,
    NoRedZone,
    NoReturn,
    NoUnwind,
    OptimizeForSize,
    OptimizeNone,
    ReadNone,
    ReadOnly,
    ReturnsTwice,
    SafeStack,
    SanitizeAddress,
    SanitizeMemory,
    SanitizeThread,
    StackProtect,
    StackProtectReq,
    StackProtectStrong,
    UWTable,
    WillReturn,
    WriteOnly,

    // Additional attributes
    Speculatable,
    StrictFP,
    InaccessibleMemOnly,
    InaccessibleMemOrArgMemOnly,
    ArgMemOnly,
    Convergent,
    NoRecurse,
    NoSync,
    NoFree,
    NullPointerIsValid,
    MustProgress,
}

/// Parameter and return value attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParameterAttribute {
    // Type attributes
    ZExt,
    SExt,
    InReg,
    ByVal,
    InAlloca,
    SRet,
    Align(u32),
    NoAlias,
    NoCapture,
    Nest,
    Returned,
    NonNull,
    Dereferenceable(u64),
    DereferenceableOrNull(u64),
    SwiftSelf,
    SwiftError,
    ImmArg,
    NoUndef,
}

/// Calling conventions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallingConvention {
    C = 0,
    Fast = 8,
    Cold = 9,
    GHC = 10,
    HiPE = 11,
    WebKitJS = 12,
    AnyReg = 13,
    PreserveMost = 14,
    PreserveAll = 15,
    Swift = 16,
    CXXFASTTLS = 17,
    X86StdCall = 64,
    X86FastCall = 65,
    ARMAPCS = 66,
    ARMAAPCS = 67,
    ARMAAPCSVFP = 68,
    MSP430INTR = 69,
    X86ThisCall = 70,
    PTXKernel = 71,
    PTXDevice = 72,
    SPIRFUNC = 75,
    SPIRKERNEL = 76,
    IntelOCLBI = 77,
    X8664SysV = 78,
    Win64 = 79,
    X86VectorCall = 80,
    HHVM = 81,
    HHVMC = 82,
    X86INTR = 83,
    AVRINTR = 84,
    AVRSIGNAL = 85,
    AVRBUILTIN = 86,
    AMDGPUVS = 87,
    AMDGPUGS = 88,
    AMDGPUPS = 89,
    AMDGPUCS = 90,
    AMDGPUKERNEL = 91,
    X86RegCall = 92,
}

/// Complex string attributes with parameters
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringAttribute {
    /// allockind("alloc"), allockind("realloc,zeroed"), etc.
    AllocKind(Vec<String>),
    /// allocsize(0), allocsize(0, 1)
    AllocSize(Vec<usize>),
    /// align(8)
    Align(u64),
    /// dereferenceable(16)
    Dereferenceable(u64),
    /// dereferenceable_or_null(16)
    DereferenceableOrNull(u64),
    /// Generic string attribute with optional value
    Generic(String, Option<String>),
}

/// A set of function attributes
#[derive(Clone, Default)]
pub struct FunctionAttributeSet {
    attributes: HashSet<FunctionAttribute>,
    string_attributes: Vec<StringAttribute>,
}

impl FunctionAttributeSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, attr: FunctionAttribute) {
        self.attributes.insert(attr);
    }

    pub fn add_string_attribute(&mut self, attr: StringAttribute) {
        self.string_attributes.push(attr);
    }

    pub fn contains(&self, attr: &FunctionAttribute) -> bool {
        self.attributes.contains(attr)
    }

    pub fn has_string_attribute(&self, name: &str) -> bool {
        self.string_attributes.iter().any(|a| {
            match a {
                StringAttribute::AllocKind(_) => name == "allockind",
                StringAttribute::AllocSize(_) => name == "allocsize",
                StringAttribute::Align(_) => name == "align",
                StringAttribute::Dereferenceable(_) => name == "dereferenceable",
                StringAttribute::DereferenceableOrNull(_) => name == "dereferenceable_or_null",
                StringAttribute::Generic(n, _) => n == name,
            }
        })
    }

    pub fn get_string_attribute(&self, name: &str) -> Option<&StringAttribute> {
        self.string_attributes.iter().find(|a| {
            match a {
                StringAttribute::AllocKind(_) => name == "allockind",
                StringAttribute::AllocSize(_) => name == "allocsize",
                StringAttribute::Align(_) => name == "align",
                StringAttribute::Dereferenceable(_) => name == "dereferenceable",
                StringAttribute::DereferenceableOrNull(_) => name == "dereferenceable_or_null",
                StringAttribute::Generic(n, _) => n == name,
            }
        })
    }

    pub fn string_attributes(&self) -> &[StringAttribute] {
        &self.string_attributes
    }

    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty() && self.string_attributes.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &FunctionAttribute> {
        self.attributes.iter()
    }
}

/// A set of parameter attributes
#[derive(Clone, Default)]
pub struct ParameterAttributeSet {
    attributes: HashSet<ParameterAttribute>,
}

impl ParameterAttributeSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, attr: ParameterAttribute) {
        self.attributes.insert(attr);
    }

    pub fn contains(&self, attr: &ParameterAttribute) -> bool {
        self.attributes.contains(attr)
    }

    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ParameterAttribute> {
        self.attributes.iter()
    }
}

impl fmt::Display for FunctionAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::AlwaysInline => "alwaysinline",
            Self::Cold => "cold",
            Self::Hot => "hot",
            Self::InlineHint => "inlinehint",
            Self::MinSize => "minsize",
            Self::Naked => "naked",
            Self::NoBuiltin => "nobuiltin",
            Self::NoDuplicate => "noduplicate",
            Self::NoImplicitFloat => "noimplicitfloat",
            Self::NoInline => "noinline",
            Self::NonLazyBind => "nonlazybind",
            Self::NoRedZone => "noredzone",
            Self::NoReturn => "noreturn",
            Self::NoUnwind => "nounwind",
            Self::OptimizeForSize => "optsize",
            Self::OptimizeNone => "optnone",
            Self::ReadNone => "readnone",
            Self::ReadOnly => "readonly",
            Self::ReturnsTwice => "returns_twice",
            Self::SafeStack => "safestack",
            Self::SanitizeAddress => "sanitize_address",
            Self::SanitizeMemory => "sanitize_memory",
            Self::SanitizeThread => "sanitize_thread",
            Self::StackProtect => "ssp",
            Self::StackProtectReq => "sspreq",
            Self::StackProtectStrong => "sspstrong",
            Self::UWTable => "uwtable",
            Self::WillReturn => "willreturn",
            Self::WriteOnly => "writeonly",
            Self::Speculatable => "speculatable",
            Self::StrictFP => "strictfp",
            Self::InaccessibleMemOnly => "inaccessiblememonly",
            Self::InaccessibleMemOrArgMemOnly => "inaccessiblemem_or_argmemonly",
            Self::ArgMemOnly => "argmemonly",
            Self::Convergent => "convergent",
            Self::NoRecurse => "norecurse",
            Self::NoSync => "nosync",
            Self::NoFree => "nofree",
            Self::NullPointerIsValid => "null_pointer_is_valid",
            Self::MustProgress => "mustprogress",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for ParameterAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZExt => write!(f, "zeroext"),
            Self::SExt => write!(f, "signext"),
            Self::InReg => write!(f, "inreg"),
            Self::ByVal => write!(f, "byval"),
            Self::InAlloca => write!(f, "inalloca"),
            Self::SRet => write!(f, "sret"),
            Self::Align(n) => write!(f, "align {}", n),
            Self::NoAlias => write!(f, "noalias"),
            Self::NoCapture => write!(f, "nocapture"),
            Self::Nest => write!(f, "nest"),
            Self::Returned => write!(f, "returned"),
            Self::NonNull => write!(f, "nonnull"),
            Self::Dereferenceable(n) => write!(f, "dereferenceable({})", n),
            Self::DereferenceableOrNull(n) => write!(f, "dereferenceable_or_null({})", n),
            Self::SwiftSelf => write!(f, "swiftself"),
            Self::SwiftError => write!(f, "swifterror"),
            Self::ImmArg => write!(f, "immarg"),
            Self::NoUndef => write!(f, "noundef"),
        }
    }
}

impl fmt::Display for CallingConvention {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::C => write!(f, "ccc"),
            Self::Fast => write!(f, "fastcc"),
            Self::Cold => write!(f, "coldcc"),
            Self::GHC => write!(f, "ghccc"),
            Self::HiPE => write!(f, "hipecc"),
            Self::WebKitJS => write!(f, "webkit_jscc"),
            Self::AnyReg => write!(f, "anyregcc"),
            Self::PreserveMost => write!(f, "preserve_mostcc"),
            Self::PreserveAll => write!(f, "preserve_allcc"),
            Self::Swift => write!(f, "swiftcc"),
            Self::CXXFASTTLS => write!(f, "cxx_fast_tlscc"),
            Self::X86StdCall => write!(f, "x86_stdcallcc"),
            Self::X86FastCall => write!(f, "x86_fastcallcc"),
            _ => write!(f, "cc {}", *self as u32),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_attribute_set() {
        let mut attrs = FunctionAttributeSet::new();
        assert!(attrs.is_empty());

        attrs.add(FunctionAttribute::NoInline);
        assert!(attrs.contains(&FunctionAttribute::NoInline));
        assert!(!attrs.is_empty());
    }

    #[test]
    fn test_parameter_attribute_set() {
        let mut attrs = ParameterAttributeSet::new();
        attrs.add(ParameterAttribute::NoAlias);
        assert!(attrs.contains(&ParameterAttribute::NoAlias));
    }

    #[test]
    fn test_function_attribute_display() {
        assert_eq!(format!("{}", FunctionAttribute::NoInline), "noinline");
        assert_eq!(format!("{}", FunctionAttribute::AlwaysInline), "alwaysinline");
    }

    #[test]
    fn test_parameter_attribute_display() {
        assert_eq!(format!("{}", ParameterAttribute::NoAlias), "noalias");
        assert_eq!(format!("{}", ParameterAttribute::Align(8)), "align 8");
    }
}
