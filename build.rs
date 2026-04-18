fn main() {
    println!("cargo:rerun-if-changed=src/bindings/csharp.rs");

    csbindgen::Builder::default()
        .input_extern_file("src/bindings/csharp.rs")
        .csharp_dll_name("xcelerate")
        .csharp_class_name("NativeMethods")
        .csharp_namespace("Xcelerate")
        .csharp_class_accessibility("public")
        .csharp_use_function_pointer(true)
        .generate_csharp_file("src/bindings/csharp/NativeMethods.g.cs")
        .unwrap();
}
