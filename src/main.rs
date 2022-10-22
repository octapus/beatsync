use std::{env, path::Path};

const USAGE: &str = "Usage: beatsync <file.wav>";

fn main() {
	// Can be changed to let file ... else {} once let else is stable
	let file = match env::args().nth(1) {
		Some(f) => f,
		None => {
			println!("{}", USAGE);
			return;
		}
	};
	let file = Path::new(&file);
	if !file.is_file() {
		println!("{}", USAGE);
		return;
	}
	println!("File: {:?}", file)
}
