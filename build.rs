extern crate bindgen;
extern crate cc;
extern crate git2;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use cc::Build;
use git2::{Repository};

fn main() {
  let repo = Repository::open(".").expect("Open git repository fail");
  let submodule = repo.find_submodule("skia").expect("Can not find git submodule skia");
  let skia_head_id = submodule.index_id().expect("Can not find commit index in git submodule skia");
  let commit_sha_short = format!("{:?}", skia_head_id);
  let commit_sha_short = &commit_sha_short.get(0..10).unwrap();

  let platform = if cfg!(target_os = "windows") {
    "win"
  } else if cfg!(target_os = "linux") {
    "linux"
  } else if cfg!(target_os = "macos") {
    "osx"
  } else {
    panic!("Unsupport platform");
  };

  let tar_name = format!("skia-static-{}.tgz", platform);

  if fs::metadata("./static/libskia.a").is_err() {
    assert!(Command::new("wget")
      .arg(&format!("https://github.com/rust-skia/skia/releases/download/{}/{}", commit_sha_short, tar_name))
      .stdin(Stdio::inherit())
      .stderr(Stdio::inherit())
      .status().unwrap().success());

    assert!(
      Command::new("tar")
        .args(&["-xvf", &tar_name])
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status().unwrap().success()
    );
  }

  let mut skia_out_dir = env::current_dir().unwrap();
  skia_out_dir.push("static");
  let skia_out_dir = skia_out_dir.to_str().unwrap();
  let current_dir = env::current_dir().unwrap();
  let current_dir_name = current_dir.to_str().unwrap();

  println!("cargo:rustc-link-search={}", &skia_out_dir);
  println!("cargo:rustc-link-lib=static=skia");
  println!("cargo:rustc-link-lib=static=skiabinding");

  let target = env::var("TARGET").unwrap();
  if target.contains("unknown-linux-gnu") {
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=bz2");
    println!("cargo:rustc-link-lib=GL");
    println!("cargo:rustc-link-lib=fontconfig");
    println!("cargo:rustc-link-lib=freetype");
  } else if target.contains("eabi") {
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=GLESv2");
  } else if target.contains("apple-darwin") {
    println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-link-lib=framework=OpenGL");
    println!("cargo:rustc-link-lib=framework=ApplicationServices");
  } else if target.contains("windows") {
    if target.contains("gnu") {
      println!("cargo:rustc-link-lib=stdc++");
    }
    println!("cargo:rustc-link-lib=usp10");
    println!("cargo:rustc-link-lib=ole32");
    println!("cargo:rustc-link-lib=user32");

    // required since GrContext::MakeVulkan is linked.
    if cfg!(feature="vulkan") {
      println!("cargo:rustc-link-lib=opengl32");
    }
  }

  // regenerate bindings?
  //
  // The bindings are generated into the src directory to support
  // IDE based symbol lookup in dependent projects, but this has the consequence
  // that the IDE and corgo might be confused by its datestamp, so we
  // avoid the regeneration if possible by implementing our own dependency checks.
  // The results of this is hard to reproduce. What can be said is that CLion's
  // cargo check invocation does from time to time takes a lot longer than expected
  // in dependent projects even though the bindings were not updated.

  let regenerate_bindings = {
    let generated_bindings = PathBuf::from("src/bindings.rs");
    if !generated_bindings.exists() { true } else {

      let skia_lib_filename =
          if cfg!(windows) { "skia.lib" } else { "libskia.a" };

      let skia_lib = PathBuf::from(&skia_out_dir).join(skia_lib_filename);
      let bindings_cpp_src = PathBuf::from("src/bindings.cpp");
      let us = PathBuf::from("build.rs");
      let config = PathBuf::from("Cargo.toml");

      fn mtime(path: &Path) -> std::time::SystemTime {
        fs::metadata(path).unwrap().modified().unwrap()
      }

      let gen_time = mtime(&generated_bindings);

      mtime(&config) > gen_time
      || mtime(&skia_lib) > gen_time
      || mtime(&bindings_cpp_src) > gen_time
      || mtime(&us) > gen_time
    }
  };

  if regenerate_bindings {
    bindgen_gen(&current_dir_name, &skia_out_dir)
  }
}

fn bindgen_gen(current_dir_name: &str, skia_out_dir: &str) {

  let mut builder = bindgen::Builder::default()
    .generate_inline_functions(true)

    .whitelist_function("C_.*")
    .whitelist_function("SkColorTypeBytesPerPixel")
    .whitelist_function("SkColorTypeIsAlwaysOpaque")
    .whitelist_function("SkColorTypeValidateAlphaType")
    .whitelist_type("SkColorSpacePrimaries")
    .whitelist_type("SkVector4")

    .rustified_enum("GrMipMapped")
    .rustified_enum("GrSurfaceOrigin")
    .rustified_enum("SkPaint_Style")
    .rustified_enum("SkPaint_Cap")
    .rustified_enum("SkPaint_Join")
    .rustified_enum("SkGammaNamed")
    .rustified_enum("SkColorSpace_RenderTargetGamma")
    .rustified_enum("SkColorSpace_Gamut")
    .rustified_enum("SkMatrix44_TypeMask")
    .rustified_enum("SkMatrix_TypeMask")
    .rustified_enum("SkMatrix_ScaleToFit")
    .rustified_enum("SkAlphaType")
    .rustified_enum("SkColorType")
    .rustified_enum("SkYUVColorSpace")
    .rustified_enum("SkPixelGeometry")
    .rustified_enum("SkSurfaceProps_Flags")
    .rustified_enum("SkBitmap_AllocFlags")
    .rustified_enum("SkImage_BitDepth")
    .rustified_enum("SkImage_CachingHint")
    .rustified_enum("SkColorChannel")
    .rustified_enum("SkYUVAIndex_Index")
    .rustified_enum("SkEncodedImageFormat")
    .rustified_enum("SkRRect_Type")
    .rustified_enum("SkRRect_Corner")
    .rustified_enum("SkRegion_Op")
    .rustified_enum("SkFont_Edging")
    .rustified_enum("SkFontMetrics_FontMetricsFlags")
    .rustified_enum("SkTypeface_SerializeBehavior")
    .rustified_enum("SkTypeface_Encoding")
    .rustified_enum("SkFontStyle_Weight")
    .rustified_enum("SkFontStyle_Width")
    .rustified_enum("SkFontStyle_Slant")

    .whitelist_var("SK_Color.*")

    .use_core()
    .clang_arg("-std=c++14");

  let mut cc_build = Build::new();

  builder = builder.header("src/bindings.cpp");

  for include_dir in fs::read_dir("skia/include").expect("Unable to read skia/include") {
    let dir = include_dir.unwrap();
    let include_path = format!("{}/{}", &current_dir_name, &dir.path().to_str().unwrap());
    builder = builder.clang_arg(format!("-I{}", &include_path));
    cc_build.include(&include_path);
  }

  if cfg!(feature="vulkan") {
	  builder = builder
      .rustified_enum("VkImageTiling")
      .rustified_enum("VkImageLayout")
      .rustified_enum("VkFormat");
	
    cc_build.define("SK_VULKAN", "1");
    builder = builder.clang_arg("-DSK_VULKAN");
    cc_build.define("SKIA_IMPLEMENTATION", "1");
    builder = builder.clang_arg("-DSKIA_IMPLEMENTATION=1");
  }

  cc_build
    .cpp(true)
    .flag("-std=c++14")
    .file("src/bindings.cpp")
    .out_dir(skia_out_dir)
    .compile("skiabinding");

  let bindings = builder.time_phases(false).generate().expect("Unable to generate bindings");

  let out_path = PathBuf::from("src");
  bindings
    .write_to_file(out_path.join("lib.rs"))
    .expect("Couldn't write bindings!");
}
