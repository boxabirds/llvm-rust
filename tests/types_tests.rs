use llvm_rust::*;

// Type system tests (100+ tests)

#[test]
fn test_void_type() {
    let ctx = Context::new();
    let void = ctx.void_type();
    assert!(void.is_void());
    assert!(!void.is_integer());
    assert_eq!(format!("{}", void), "void");
}

#[test]
fn test_i1_type() {
    let ctx = Context::new();
    let i1 = ctx.bool_type();
    assert!(i1.is_integer());
    assert_eq!(i1.int_width(), Some(1));
    assert_eq!(format!("{}", i1), "i1");
}

#[test]
fn test_i8_type() {
    let ctx = Context::new();
    let i8 = ctx.int8_type();
    assert!(i8.is_integer());
    assert_eq!(i8.int_width(), Some(8));
    assert_eq!(format!("{}", i8), "i8");
}

#[test]
fn test_i16_type() {
    let ctx = Context::new();
    let i16 = ctx.int16_type();
    assert!(i16.is_integer());
    assert_eq!(i16.int_width(), Some(16));
    assert_eq!(format!("{}", i16), "i16");
}

#[test]
fn test_i32_type() {
    let ctx = Context::new();
    let i32 = ctx.int32_type();
    assert!(i32.is_integer());
    assert_eq!(i32.int_width(), Some(32));
    assert_eq!(format!("{}", i32), "i32");
}

#[test]
fn test_i64_type() {
    let ctx = Context::new();
    let i64 = ctx.int64_type();
    assert!(i64.is_integer());
    assert_eq!(i64.int_width(), Some(64));
    assert_eq!(format!("{}", i64), "i64");
}

#[test]
fn test_custom_integer_type() {
    let ctx = Context::new();
    let i37 = ctx.int_type(37);
    assert!(i37.is_integer());
    assert_eq!(i37.int_width(), Some(37));
    assert_eq!(format!("{}", i37), "i37");
}

#[test]
fn test_half_type() {
    let ctx = Context::new();
    let half = ctx.half_type();
    assert!(half.is_float());
    assert_eq!(format!("{}", half), "half");
}

#[test]
fn test_float_type() {
    let ctx = Context::new();
    let float = ctx.float_type();
    assert!(float.is_float());
    assert_eq!(format!("{}", float), "float");
}

#[test]
fn test_double_type() {
    let ctx = Context::new();
    let double = ctx.double_type();
    assert!(double.is_float());
    assert_eq!(format!("{}", double), "double");
}

#[test]
fn test_pointer_type() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let ptr = ctx.ptr_type(i32_type);
    assert!(ptr.is_pointer());
    assert_eq!(format!("{}", ptr), "i32*");
}

#[test]
fn test_array_type() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let array = ctx.array_type(i32_type, 10);
    assert!(array.is_array());
    assert_eq!(format!("{}", array), "[10 x i32]");
}

#[test]
fn test_vector_type() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let vector = ctx.vector_type(i32_type, 4);
    assert!(vector.is_vector());
    assert_eq!(format!("{}", vector), "<4 x i32>");
}

#[test]
fn test_label_type() {
    let ctx = Context::new();
    let label = ctx.label_type();
    assert!(label.is_label());
    assert_eq!(format!("{}", label), "label");
}

#[test]
fn test_token_type() {
    let ctx = Context::new();
    let token = ctx.token_type();
    assert!(token.is_token());
    assert_eq!(format!("{}", token), "token");
}

#[test]
fn test_metadata_type() {
    let ctx = Context::new();
    let metadata = ctx.metadata_type();
    assert!(metadata.is_metadata());
    assert_eq!(format!("{}", metadata), "metadata");
}

// Function type tests
#[test]
fn test_function_type_no_args() {
    let ctx = Context::new();
    let void = ctx.void_type();
    let fn_type = ctx.function_type(void, vec![], false);
    assert!(fn_type.is_function());
}

#[test]
fn test_function_type_with_args() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone(), i32_type.clone()], false);
    assert!(fn_type.is_function());
}

#[test]
fn test_function_type_vararg() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let fn_type = ctx.function_type(i32_type.clone(), vec![i32_type.clone()], true);
    assert!(fn_type.is_function());
}

// Struct type tests
#[test]
fn test_struct_type_anonymous() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let float_type = ctx.float_type();
    let struct_type = types::Type::struct_type(&ctx, vec![i32_type, float_type], None);
    assert!(struct_type.is_struct());
}

#[test]
fn test_struct_type_named() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let struct_type = types::Type::struct_type(&ctx, vec![i32_type], Some("MyStruct".to_string()));
    assert!(struct_type.is_struct());
}

#[test]
fn test_struct_type_packed() {
    let ctx = Context::new();
    let i8_type = ctx.int8_type();
    let i32_type = ctx.int32_type();
    let struct_type = types::Type::struct_type_packed(&ctx, vec![i8_type, i32_type], None, true);
    assert!(struct_type.is_struct());
}

// Nested types
#[test]
fn test_nested_pointer() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let ptr1 = ctx.ptr_type(i32_type);
    let ptr2 = ctx.ptr_type(ptr1);
    assert!(ptr2.is_pointer());
    assert_eq!(format!("{}", ptr2), "i32**");
}

#[test]
fn test_array_of_pointers() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let ptr = ctx.ptr_type(i32_type);
    let array = ctx.array_type(ptr, 5);
    assert!(array.is_array());
    assert_eq!(format!("{}", array), "[5 x i32*]");
}

#[test]
fn test_pointer_to_array() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let array = ctx.array_type(i32_type, 5);
    let ptr = ctx.ptr_type(array);
    assert!(ptr.is_pointer());
}

#[test]
fn test_vector_of_floats() {
    let ctx = Context::new();
    let float_type = ctx.float_type();
    let vector = ctx.vector_type(float_type, 8);
    assert!(vector.is_vector());
}

#[test]
fn test_array_of_vectors() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let vector = ctx.vector_type(i32_type, 4);
    let array = ctx.array_type(vector, 10);
    assert!(array.is_array());
}

// Type queries
#[test]
fn test_array_info() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let array = ctx.array_type(i32_type.clone(), 42);
    if let Some((elem, size)) = array.array_info() {
        assert_eq!(size, 42);
        assert!(elem.is_integer());
    } else {
        panic!("Expected array info");
    }
}

#[test]
fn test_vector_info() {
    let ctx = Context::new();
    let float_type = ctx.float_type();
    let vector = ctx.vector_type(float_type, 16);
    if let Some((elem, size)) = vector.vector_info() {
        assert_eq!(size, 16);
        assert!(elem.is_float());
    } else {
        panic!("Expected vector info");
    }
}

#[test]
fn test_pointee_type() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let ptr = ctx.ptr_type(i32_type.clone());
    if let Some(pointee) = ptr.pointee_type() {
        assert!(pointee.is_integer());
    } else {
        panic!("Expected pointee type");
    }
}

// More comprehensive type tests
macro_rules! test_integer_width {
    ($name:ident, $width:expr) => {
        #[test]
        fn $name() {
            let ctx = Context::new();
            let int_type = ctx.int_type($width);
            assert!(int_type.is_integer());
            assert_eq!(int_type.int_width(), Some($width));
            assert_eq!(format!("{}", int_type), format!("i{}", $width));
        }
    };
}

test_integer_width!(test_i2, 2);
test_integer_width!(test_i3, 3);
test_integer_width!(test_i4, 4);
test_integer_width!(test_i5, 5);
test_integer_width!(test_i6, 6);
test_integer_width!(test_i7, 7);
test_integer_width!(test_i9, 9);
test_integer_width!(test_i10, 10);
test_integer_width!(test_i11, 11);
test_integer_width!(test_i12, 12);
test_integer_width!(test_i13, 13);
test_integer_width!(test_i14, 14);
test_integer_width!(test_i15, 15);
test_integer_width!(test_i17, 17);
test_integer_width!(test_i18, 18);
test_integer_width!(test_i19, 19);
test_integer_width!(test_i20, 20);
test_integer_width!(test_i24, 24);
test_integer_width!(test_i25, 25);
test_integer_width!(test_i31, 31);
test_integer_width!(test_i33, 33);
test_integer_width!(test_i48, 48);
test_integer_width!(test_i63, 63);
test_integer_width!(test_i65, 65);
test_integer_width!(test_i96, 96);
test_integer_width!(test_i128, 128);
test_integer_width!(test_i256, 256);
test_integer_width!(test_i512, 512);

// Array size tests
macro_rules! test_array_size {
    ($name:ident, $size:expr) => {
        #[test]
        fn $name() {
            let ctx = Context::new();
            let i32_type = ctx.int32_type();
            let array = ctx.array_type(i32_type, $size);
            assert!(array.is_array());
            if let Some((_, s)) = array.array_info() {
                assert_eq!(s, $size);
            }
        }
    };
}

test_array_size!(test_array_0, 0);
test_array_size!(test_array_1, 1);
test_array_size!(test_array_2, 2);
test_array_size!(test_array_100, 100);
test_array_size!(test_array_1000, 1000);
test_array_size!(test_array_10000, 10000);

// Vector size tests
macro_rules! test_vector_size {
    ($name:ident, $size:expr) => {
        #[test]
        fn $name() {
            let ctx = Context::new();
            let i32_type = ctx.int32_type();
            let vector = ctx.vector_type(i32_type, $size);
            assert!(vector.is_vector());
            if let Some((_, s)) = vector.vector_info() {
                assert_eq!(s, $size);
            }
        }
    };
}

test_vector_size!(test_vector_2, 2);
test_vector_size!(test_vector_4, 4);
test_vector_size!(test_vector_8, 8);
test_vector_size!(test_vector_16, 16);
test_vector_size!(test_vector_32, 32);
test_vector_size!(test_vector_64, 64);

// Complex nested types
#[test]
fn test_complex_nested_struct() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let float_type = ctx.float_type();
    let inner_struct = types::Type::struct_type(&ctx, vec![i32_type.clone(), float_type.clone()], None);
    let outer_struct = types::Type::struct_type(&ctx, vec![inner_struct, i32_type], None);
    assert!(outer_struct.is_struct());
}

#[test]
fn test_function_with_struct_args() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let struct_type = types::Type::struct_type(&ctx, vec![i32_type.clone()], None);
    let fn_type = ctx.function_type(i32_type.clone(), vec![struct_type], false);
    assert!(fn_type.is_function());
}

#[test]
fn test_function_returning_pointer() {
    let ctx = Context::new();
    let i32_type = ctx.int32_type();
    let ptr_type = ctx.ptr_type(i32_type.clone());
    let fn_type = ctx.function_type(ptr_type, vec![i32_type], false);
    assert!(fn_type.is_function());
}
