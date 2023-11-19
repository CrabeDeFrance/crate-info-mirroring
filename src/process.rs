use std::fs;
use std::fs::File;
use std::io;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use threadpool::ThreadPool;

use super::model::CrateMetadata;

static DEFAULT: &str = "https://crates.io";

/// write metadata in a file
fn write_crate_metadata(path: &Path, metadata: String) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(metadata.as_bytes())?;
    Ok(())
}

/// fetches and prints package metadata from crates.io
fn get_crate_metadata(crate_name: &str) -> Result<String, String> {
    let mut req = crates_io::Registry::new(DEFAULT.to_string(), None);

    req.get_crate_data(crate_name)
        .map_err(|e| format!("Error fetching data for {}: {}", crate_name, e))
}

/// return true if metadata file is up to date. return false if we have to update metadata
fn check_metadata_file(path_in: &Path, path_out: &Path, crate_name: &str) -> io::Result<bool> {
    let file = File::open(path_out);
    if let Err(ioerr) = file {
        if ioerr.kind() != io::ErrorKind::NotFound {
            error!(
                "{}: Error: can't open file {:?}: {}",
                crate_name, path_out, ioerr
            );
        }
        debug!("{}: not found", crate_name);
        return Ok(false);
    }
    let file = file.unwrap();

    let reader = BufReader::new(file);
    let crate_data: CrateMetadata = serde_json::from_reader(reader)?;

    // now we check if there are more folders than before
    let known_versions: Vec<String> = crate_data.versions.into_iter().map(|v| v.num).collect();

    for entry in fs::read_dir(path_in)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            let e = entry_path.file_name().unwrap().to_str().unwrap();
            if !known_versions.iter().any(|v| v == e) {
                debug!("{}: version {:?} not found", crate_name, e);
                return Ok(false);
            }
        }
    }

    debug!("{}: in cache", crate_name);

    Ok(true)
}

/// update a metadata file
fn process_crate(path_in: &Path, mut path_out: PathBuf) -> io::Result<()> {
    let crate_name = String::from(path_in.file_name().unwrap().to_str().unwrap());
    let mut filename = crate_name.clone();
    filename.push_str(".json");

    // we create directory for metadata file if needed
    if fs::metadata(&path_out).is_err() {
        fs::create_dir_all(&path_out)?;
        path_out.push(&filename);
    } else {
        path_out.push(&filename);
        // or we check if we need to update the current metadata file
        let test = check_metadata_file(path_in, &path_out, &crate_name);
        match test {
            Ok(ret) => {
                if ret {
                    return Ok(());
                }
            }
            Err(err) => {
                error!("Error reading {}: {}", filename, err);
            }
        }
    }

    // if we are here, we need to write a new metadata file
    let meta = get_crate_metadata(&crate_name);
    match meta {
        Ok(ret) => {
            write_crate_metadata(&path_out, ret)?;
            info!("{}: downloaded", crate_name);
            Ok(())
        }
        Err(ret) => Err(io::Error::new(io::ErrorKind::Other, ret)),
    }
}

fn spawn_process_crate(
    pool: &ThreadPool,
    tx: Sender<u32>,
    entry_path_in: PathBuf,
    path_out: PathBuf,
) {
    pool.execute(move || {
        if let Err(ret) = process_crate(&entry_path_in, path_out) {
            error!("Can't process crate {:?}: {}", entry_path_in, ret);
        }
        tx.send(1).expect("Can't send reply");
    });
}

fn build_new_path(path: &Path, currentpath: &Path) -> PathBuf {
    let dirname = currentpath.file_name().unwrap().to_str().unwrap();

    // use shadowing to continue to build path
    let mut out_crate_path = path.to_path_buf();
    out_crate_path.push(dirname);
    out_crate_path
}

/// update all crate's metadata
pub(crate) fn parse_directory(
    dirname_in: String,
    root_dirname_out: String,
    thread_count: u8,
) -> io::Result<()> {
    let path_in = Path::new(&dirname_in);
    if !path_in.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid directory: '{}'", dirname_in),
        ));
    }

    let pool = ThreadPool::new(thread_count as _);
    let mut tasks: usize = 0;

    let (tx, rx) = channel();

    // we will build output path from input path
    let out_crate_path = PathBuf::from(root_dirname_out);

    for entry in fs::read_dir(path_in)? {
        let entry = entry?;
        let entry_path_in = entry.path();
        if entry_path_in.is_dir() {
            // use shadowing to continue to build path
            let out_crate_path = build_new_path(&out_crate_path, &entry_path_in);

            /* here we have 2 first letters of the crate's name or 1/2/3 (which is crate's name length) */
            let dirname = entry_path_in.file_name().unwrap().to_str().unwrap();
            match dirname {
                "1" | "2" => {
                    for entry in fs::read_dir(entry_path_in)? {
                        let entry = entry?;
                        let entry_path_in = entry.path();
                        if entry_path_in.is_dir() {
                            /* here we have full crate's name */
                            let out_crate_path = build_new_path(&out_crate_path, &entry_path_in);

                            spawn_process_crate(
                                &pool,
                                tx.clone(),
                                entry_path_in.to_path_buf(),
                                out_crate_path,
                            );
                            tasks += 1;
                        }
                    }
                }
                _ => {
                    for entry in fs::read_dir(entry_path_in)? {
                        let entry = entry?;
                        let entry_path_in = entry.path();
                        if entry_path_in.is_dir() {
                            // use shadowing to continue to build path
                            let out_crate_path = build_new_path(&out_crate_path, &entry_path_in);

                            /* here we have 2 next letters of the crate's name */
                            for entry in fs::read_dir(entry_path_in)? {
                                let entry = entry?;
                                let entry_path_in = entry.path();
                                if entry_path_in.is_dir() {
                                    // use shadowing to continue to build path
                                    let out_crate_path =
                                        build_new_path(&out_crate_path, &entry_path_in);

                                    /* here we have full crate's name */
                                    spawn_process_crate(
                                        &pool,
                                        tx.clone(),
                                        entry_path_in.to_path_buf(),
                                        out_crate_path,
                                    );
                                    tasks += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // wait for all returns ( ~ 80k )
    let _ret: Vec<u32> = rx.iter().take(tasks).collect();

    Ok(())
}
