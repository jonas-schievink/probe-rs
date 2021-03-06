use std::env;
use std::fs::{read_dir, read_to_string, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use probe_rs::{probe::flash::FlashAlgorithm, target::Target};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("targets.rs");
    let mut f = File::create(&dest_path).unwrap();

    // TARGETS
    let mut files = vec![];
    visit_dirs(Path::new("algorithms"), &mut files).unwrap();

    let mut algorithm_names = vec![];
    let mut algorithm_files = vec![];

    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    for file in files {
        let string = read_to_string(&file).expect(
            "Algorithm definition file could not be read. This is a bug. Please report it.",
        );

        match FlashAlgorithm::new(&string) {
            Ok(_algorithm) => {
                let abs_path = root_dir.join(&file);

                algorithm_files.push(abs_path.to_str().unwrap().to_owned());

                algorithm_names.push(
                    file.strip_prefix("algorithms")
                        .unwrap()
                        .to_str()
                        .expect("Non UTF-8 Filename!")
                        .to_owned(),
                );
            }
            Err(e) => {
                panic!("Failed to parse target file: {:?} because:\n{}", file, e);
            }
        }
    }

    dbg!(&algorithm_names);
    dbg!(&algorithm_files);

    // TARGETS
    let mut files = vec![];
    visit_dirs(Path::new("targets"), &mut files).unwrap();

    let mut target_names = vec![];
    let mut target_files = vec![];

    for file in files {
        let string = read_to_string(&file)
            .expect("Chip definition file could not be read. This is a bug. Please report it.");
        match Target::new(&string) {
            Ok(target) => {
                if let Some(algo) = target.flash_algorithm {
                    assert!(
                        algorithm_names.contains(&algo),
                        "Algorithm {} does not exist.",
                        algo
                    );
                }

                target_files.push(root_dir.join(file).to_str().unwrap().to_owned());

                target_names.push(target.name.to_ascii_lowercase());
            }
            Err(e) => {
                panic!("Failed to parse target file: {:?} because:\n{}", file, e);
            }
        }
    }

    dbg!(&target_names);
    dbg!(&target_files);

    let stream: String = format!(
        "{}",
        quote::quote! {
        // START QUOTE
            lazy_static::lazy_static! {
                static ref FLASH_ALGORITHMS: HashMap<&'static str, &'static str> = vec![
                    #((#algorithm_names, include_str!(#algorithm_files)),)*
                ].into_iter().collect();

                static ref TARGETS: HashMap<&'static str, &'static str> = vec![
                    #((#target_names, include_str!(#target_files)),)*
                ].into_iter().collect();
            }
        // END QUOTE
        }
    );

    f.write_all(stream.as_bytes())
        .expect("Writing build.rs output failed.");
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path, targets: &mut Vec<PathBuf>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, targets)?;
            } else {
                targets.push(path.to_owned());
            }
        }
    }
    Ok(())
}
