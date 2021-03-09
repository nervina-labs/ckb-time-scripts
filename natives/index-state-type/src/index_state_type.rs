extern crate alloc;

#[path = "../../../contracts/index-state-type/src/entry.rs"]
mod entry;
#[path = "../../../contracts/index-state-type/src/error.rs"]
mod error;

fn main() {
    if let Err(err) = entry::main() {
        std::process::exit(err as i32);
    }
}
