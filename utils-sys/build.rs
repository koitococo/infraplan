use std::{env, path::PathBuf};

fn main() {
  let outdir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
  let outdir = PathBuf::from(outdir);

  // This is the directory where the `c` library is located. Canonicalize the path as `rustc-link-search` requires an absolute path.
  let src_path = PathBuf::from("ffi_src").canonicalize().expect("cannot canonicalize path");

  // This is the path to the `c` headers file.
  let headers_path = src_path.join("utils.hpp");

  // This is the path to the intermediate object file for our library.
  let obj_path = outdir.join("utils.o");
  // This is the path to the static library file.
  let lib_path = outdir.join("libutils.a");

  // Tell cargo to look for shared libraries in the specified directory
  println!("cargo:rustc-link-search={}", outdir.to_str().unwrap());

  // Tell cargo to tell rustc to link our `utils` library. Cargo will automatically know it must look for a `libutils.a` file.
  println!("cargo:rustc-link-lib=utils");
  println!("cargo:rustc-link-lib=stdc++");

  // Run `clang` to compile the `utils.cpp` file into a `utils.o` object file. Panic if it is not possible to spawn the process.
  std::process::Command::new("clang++")
    .args(vec!["-c", "-x", "c++", "-std=c++17", "-fPIC", "-static", "-o"])
    .arg(&obj_path)
    .arg(src_path.join("utils.cpp"))
    .output()
    .expect("could not spawn `clang++`")
    .status
    .success()
  {
    panic!("could not compile object file");
  }

  // Run `ar` to generate the `libutils.a` file from the `utils.o` file. Panic if it is not possible to spawn the process.
  if !std::process::Command::new("ar")
    .arg("rcs")
    .arg(lib_path)
    .arg(obj_path)
    .output()
    .expect("could not spawn `ar`")
    .status
    .success()
  {
    panic!("could not emit library file");
  }

  // The bindgen::Builder is the main entry point
  // to bindgen, and lets you build up options for
  // the resulting bindings.
  let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(headers_path.to_str().expect("Path is not a valid string"))
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

  // Write the bindings to the $OUT_DIR/bindings.rs file.
  let out_path = outdir.join("bindings.rs");
  bindings.write_to_file(out_path).expect("Couldn't write bindings!");
}
