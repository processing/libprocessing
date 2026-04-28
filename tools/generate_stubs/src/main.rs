use pyo3_introspection::model::{Argument, Arguments, Expr, Function, Module};
use pyo3_introspection::{introspect_cdylib, module_stub_files};
use std::path::{Path, PathBuf};
use std::{env, fs};

const SWIZZLE_CHARS: [char; 4] = ['x', 'y', 'z', 'w'];

fn swizzle_props(dim: usize) -> Vec<Function> {
    let chars = &SWIZZLE_CHARS[..dim];
    let mut out = Vec::new();
    for length in 2..=4 {
        let count = chars.len().pow(length as u32);
        for n in 0..count {
            let mut name = String::with_capacity(length);
            let mut idx = n;
            for _ in 0..length {
                name.push(chars[idx % chars.len()]);
                idx /= chars.len();
            }
            out.push(Function {
                name,
                decorators: vec![Expr::Name {
                    id: "property".into(),
                }],
                arguments: Arguments {
                    positional_only_arguments: vec![Argument {
                        name: "self".into(),
                        default_value: None,
                        annotation: None,
                    }],
                    arguments: vec![],
                    vararg: None,
                    keyword_only_arguments: vec![],
                    kwarg: None,
                },
                returns: Some(Expr::Attribute {
                    value: Box::new(Expr::Name {
                        id: "mewnala.math".into(),
                    }),
                    attr: format!("Vec{length}"),
                }),
                is_async: false,
                docstring: None,
            });
        }
    }
    out
}

fn inject_swizzles(module: &mut Module) {
    for math in module.modules.iter_mut().filter(|m| m.name == "math") {
        for cls in math.classes.iter_mut() {
            let dim = match cls.name.as_str() {
                "Vec2" => 2,
                "Vec3" => 3,
                "Vec4" => 4,
                _ => continue,
            };
            let existing: std::collections::HashSet<String> =
                cls.methods.iter().map(|m| m.name.clone()).collect();
            cls.methods.extend(
                swizzle_props(dim)
                    .into_iter()
                    .filter(|m| !existing.contains(&m.name)),
            );
        }
    }
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn find_cdylib() -> PathBuf {
    let target_dir = workspace_root().join("target").join("release");

    // Platform-specific library name
    let lib_name = if cfg!(target_os = "macos") {
        "libmewnala.dylib"
    } else if cfg!(target_os = "windows") {
        "mewnala.dll"
    } else {
        "libmewnala.so"
    };

    let path = target_dir.join(lib_name);
    if !path.exists() {
        eprintln!("Could not find {}", path.display());
        eprintln!("Make sure to build processing_pyo3 first:");
        eprintln!("  cargo build --release -p processing_pyo3");
        std::process::exit(1);
    }
    path
}

fn main() {
    let cdylib_path = if let Some(path) = env::args().nth(1) {
        PathBuf::from(path)
    } else {
        find_cdylib()
    };

    eprintln!("Introspecting: {}", cdylib_path.display());

    let mut module =
        introspect_cdylib(&cdylib_path, "mewnala").expect("Failed to introspect cdylib");

    module.incomplete = false;
    inject_swizzles(&mut module);

    let mut stubs = module_stub_files(&module);

    // join in extras

    let extras_dir = workspace_root()
        .join("crates")
        .join("processing_pyo3")
        .join("stubs");
    if extras_dir.is_dir() {
        for entry in fs::read_dir(&extras_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("pyi") {
                continue;
            }
            let filename = path.file_name().unwrap().to_owned();
            let extra = fs::read_to_string(&path).unwrap();
            let target = stubs.entry(PathBuf::from(&filename)).or_default();
            if !target.is_empty() && !target.ends_with('\n') {
                target.push('\n');
            }
            target.push('\n');
            target.push_str(&extra);
            eprintln!("Appended extras: {}", path.display());
        }
    }

    let output_dir = workspace_root()
        .join("crates")
        .join("processing_pyo3")
        .join("mewnala");

    for (filename, content) in &stubs {
        let out_path = output_dir.join(filename);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&out_path, content).unwrap();
        eprintln!("Wrote: {}", out_path.display());
    }

    eprintln!("Done! Generated {} stub file(s)", stubs.len());
}
