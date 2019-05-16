use std::env;
use std::fs::canonicalize;
use std::path::PathBuf;
use std::process::Command;

// #[cfg(debug_assertions)]
// static TARGET: &'static str = "debug"
// #[cfg(not(debug_assertions))]
// static APP_JS: &'static str = include_str!("webui/build/min.js");

fn main() {
    let project_dir = canonicalize(PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())).unwrap();

    assert!(Command::new("make")
        //.env("LUA_DIR", lua_dir)
        //.args(&["-f", "src/lua/luacall_linux.mak"])
        .args(&[if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }])
        .current_dir(project_dir.join("src/webui"))
        .status()
        .expect("failed to make javascript app")
        .success());
}
