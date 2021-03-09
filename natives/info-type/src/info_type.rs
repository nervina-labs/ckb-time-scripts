extern crate alloc;

#[path = "../../../contracts/info-type/src/entry.rs"]
mod entry;
#[path = "../../../contracts/info-type/src/error.rs"]
mod error;

fn main() {
    if let Err(err) = entry::main() {
        std::process::exit(err as i32);
    }
}
