use hashbrown::HashSet;
use lazy_static::lazy_static;
use libloading::Library;
use notify::{immediate_watcher, EventKind, RecursiveMode, Watcher};
use std::{
    process::{Command, Stdio},
    sync::{mpsc, Mutex},
    thread,
    time::Duration,
};

use find_folder::Search;
use std::{env, path::PathBuf};
use tracing::{debug, error, info};

#[cfg(target_os = "windows")]
const COMPILED_FILE: &str = "voxygen_anim.dll";
#[cfg(target_os = "windows")]
const ACTIVE_FILE: &str = "voxygen_anim_active.dll";

#[cfg(not(target_os = "windows"))]
const COMPILED_FILE: &str = "libvoxygen_anim.so";
#[cfg(not(target_os = "windows"))]
const ACTIVE_FILE: &str = "libvoxygen_anim_active.so";

// This option is required as `hotreload()` moves the `LoadedLib`.
lazy_static! {
    pub static ref LIB: Mutex<Option<LoadedLib>> = Mutex::new(Some(LoadedLib::compile_load()));
}

/// LoadedLib holds a loaded dynamic library and the location of library file
/// with the appropriate OS specific name and extension i.e.
/// `libvoxygen_anim_active.dylib`, `voxygen_anim_active.dll`.
///
/// # NOTE
/// DOES NOT WORK ON MACOS, due to some limitations with hot-reloading the
/// `.dylib`.
pub struct LoadedLib {
    /// Loaded library.
    pub lib: Library,
    /// Path to the library.
    pub lib_path: PathBuf,
}

impl LoadedLib {
    /// Compile and load the dynamic library
    ///
    /// This is necessary because the very first time you use hot reloading you
    /// wont have the library, so you can't load it until you have compiled it!
    fn compile_load() -> Self {
        #[cfg(target_os = "macos")]
        error!("The hot reloading feature does not work on macos.");

        // Compile
        if !compile() {
            panic!("Animation compile failed.");
        } else {
            info!("Animation compile succeeded.");
        }

        copy(&LoadedLib::determine_path());

        Self::load()
    }

    /// Load a library from disk.
    ///
    /// Currently this is pretty fragile, it gets the path of where it thinks
    /// the dynamic library should be and tries to load it. It will panic if it
    /// is missing.
    fn load() -> Self {
        let lib_path = LoadedLib::determine_path();

        // Try to load the library.
        let lib = match Library::new(lib_path.clone()) {
            Ok(lib) => lib,
            Err(e) => panic!(
                "Tried to load dynamic library from {:?}, but it could not be found. The first \
                 reason might be that you need to uncomment a line in `voxygen/anim/Cargo.toml` \
                 to build the library required for hot reloading. The second is we may require a \
                 special case for your OS so we can find it. {:?}",
                lib_path, e
            ),
        };

        Self { lib, lib_path }
    }

    /// Determine the path to the dynamic library based on the path of the
    /// current executable.
    fn determine_path() -> PathBuf {
        let current_exe = env::current_exe();

        // If we got the current_exe, we need to go up a level and then down
        // in to debug (in case we were in release or another build dir).
        let mut lib_path = match current_exe {
            Ok(mut path) => {
                // Remove the filename to get the directory.
                path.pop();

                // Search for the debug directory.
                let dir = Search::ParentsThenKids(1, 1)
                    .of(path)
                    .for_folder("debug")
                    .expect(
                        "Could not find the debug build directory relative to the current \
                         executable.",
                    );

                debug!(?dir, "Found the debug build directory.");
                dir
            },
            Err(e) => {
                panic!(
                    "Could not determine the path of the current executable, this is needed to \
                     hotreload the dynamic library. {:?}",
                    e
                );
            },
        };

        // Determine the platform specific path and push it onto our already
        // established target/debug dir.
        lib_path.push(ACTIVE_FILE);

        lib_path
    }
}

/// Initialise a watcher.
///
/// The assumption is that this is run from the voxygen crate's root directory
/// as it will watch the relative path `anim` for any changes to `.rs`
/// files. Upon noticing changes it will wait a moment and then recompile.
pub fn init() {
    // Make sure first compile is done by accessing the lazy_static and then
    // immediately dropping (because we don't actually need it).
    drop(LIB.lock());

    // TODO: use crossbeam
    let (reload_send, reload_recv) = mpsc::channel();

    // Start watcher
    let mut watcher = immediate_watcher(move |res| event_fn(res, &reload_send)).unwrap();
    watcher.watch("anim", RecursiveMode::Recursive).unwrap();

    // Start reloader that watcher signals
    // "Debounces" events since I can't find the option to do this in the latest
    // `notify`
    thread::spawn(move || {
        let mut modified_paths = HashSet::new();

        while let Ok(path) = reload_recv.recv() {
            modified_paths.insert(path);
            // Wait for any additional modify events before reloading
            while let Ok(path) = reload_recv.recv_timeout(Duration::from_millis(300)) {
                modified_paths.insert(path);
            }

            info!(
                ?modified_paths,
                "Hot reloading animations because files in `anim` modified."
            );

            hotreload();
        }
    });

    // Let the watcher live forever
    std::mem::forget(watcher);
}

/// Event function to hotreload the dynamic library
///
/// This is called by the watcher to filter for modify events on `.rs` files
/// before sending them back.
fn event_fn(res: notify::Result<notify::Event>, sender: &mpsc::Sender<String>) {
    match res {
        Ok(event) => match event.kind {
            EventKind::Modify(_) => {
                event
                    .paths
                    .iter()
                    .filter(|p| p.extension().map(|e| e == "rs").unwrap_or(false))
                    .map(|p| p.to_string_lossy().into_owned())
                    // Signal reloader
                    .for_each(|p| { let _ = sender.send(p); });
            },
            _ => {},
        },
        Err(e) => error!(?e, "Animation hotreload watcher error."),
    }
}

/// Hotreload the dynamic library
///
/// This will reload the dynamic library by first internally calling compile
/// and then reloading the library.
fn hotreload() {
    // Do nothing if recompile failed.
    if compile() {
        let mut lock = LIB.lock().unwrap();

        // Close lib.
        let loaded_lib = lock.take().unwrap();
        loaded_lib.lib.close().unwrap();
        copy(&loaded_lib.lib_path);

        // Open new lib.
        *lock = Some(LoadedLib::load());

        info!("Updated animations.");
    }
}

/// Recompile the anim package
///
/// Returns `false` if the compile failed.
fn compile() -> bool {
    let output = Command::new("cargo")
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .arg("build")
        .arg("--package")
        .arg("veloren-voxygen-anim")
        .output()
        .unwrap();

    output.status.success()
}

/// Copy the lib file, so we have an `_active` copy.
///
/// We do this for all OS's although it is only strictly necessary for windows.
/// The reason we do this is to make the code easier to understand and debug.
fn copy(lib_path: &PathBuf) {
    // Use the platform specific names.
    let lib_compiled_path = lib_path.with_file_name(COMPILED_FILE);
    let lib_output_path = lib_path.with_file_name(ACTIVE_FILE);

    // Get the path to where the lib was compiled to.
    debug!(?lib_compiled_path, ?lib_output_path, "Moving.");

    // Copy the library file from where it is output, to where we are going to
    // load it from i.e. lib_path.
    std::fs::copy(lib_compiled_path, lib_output_path).expect("Failed to rename dynamic library.");
}
