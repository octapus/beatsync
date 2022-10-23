use std::env;

use minifb::{Window, WindowOptions, Key};

const USAGE: &str = "Usage: beatsync <file.wav>";

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

fn main() {
	// waiting for if let chains to become stable...
	let file = match env::args_os().nth(1) {
		Some(f) => f,
		None => {
			println!("{}", USAGE);
			return;
		}
	};
	let mut reader = match hound::WavReader::open(file) {
		Ok(r) => r,
		Err(_) => {
			println!("{}", USAGE);
			return;
		}
	};

	assert_eq!(2, reader.spec().channels);
	assert_eq!(16, reader.spec().bits_per_sample);
	assert_eq!(hound::SampleFormat::Int, reader.spec().sample_format);

	for sample in reader.samples::<i16>().skip(200).take(20) {
		println!("{}", sample.unwrap());
	}


	// gui stuff
	// in future, make this buffer mut and update it in main loop
	let buffer: Vec<u32> = vec![u32::MAX; WIDTH * HEIGHT];
	let mut window = Window::new(
		"Test - ESC to exit",
		WIDTH,
		HEIGHT,
		WindowOptions::default()
	).unwrap_or_else(|e| { panic!("{}", e); });
	// 16700 for 60 fps, 6900 for 144
	window.limit_update_rate(Some(std::time::Duration::from_micros(6800)));

	// main loop
	while window.is_open() && !window.is_key_down(Key::Escape) && !window.is_key_down(Key::Q) {
		window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
	}
}
