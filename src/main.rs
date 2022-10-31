use std::{env, ffi::OsString, fs::File, io::BufReader, time::Instant};

use hound::WavReader;
use minifb::{Key, Window, WindowOptions};

const USAGE: &str = "Usage: beatsync <file.wav> [window width] [window height]";

const DEFAULT_WIDTH: usize = 1920;
const DEFAULT_HEIGHT: usize = 1080;

fn parse_args() -> Option<(WavReader<BufReader<File>>, usize, usize)> {
	let args: Vec<OsString> = env::args_os().collect();
	// waiting for if let chains to become stable...
	let reader = match hound::WavReader::open(args.get(1)?) {
		Ok(r) => r,
		Err(_) => {
			return None;
		}
	};
	let width =
		(|| args.get(2)?.to_string_lossy().trim().parse::<usize>().ok())().unwrap_or(DEFAULT_WIDTH);
	let height = (|| args.get(3)?.to_string_lossy().trim().parse::<usize>().ok())()
		.unwrap_or(DEFAULT_HEIGHT);
	Some((reader, width, height))
}

fn render(input: &[i16], width: usize, height: usize) -> Vec<u32> {
	let mut output = vec![0u32; width * height];
	let max_list = input
		.chunks_exact(input.len() / width)
		.map(|chunk| chunk.iter().fold(0, |acc, &x| std::cmp::max(acc, x.abs())));
	assert_eq!(max_list.len(), width);
	for (i, max) in max_list.enumerate() {
		for j in 0..(usize::try_from(max).unwrap() * height / usize::try_from(i16::MAX).unwrap()) {
			output[j * width + i] = u32::MAX;
		}
	}
	output
}

fn main() {
	let (reader, width, height) = match parse_args() {
		Some((r, w, h)) => (r, w, h),
		None => {
			println!("{USAGE}");
			return;
		}
	};

	assert_eq!(2, reader.spec().channels);
	assert_eq!(16, reader.spec().bits_per_sample);
	assert_eq!(hound::SampleFormat::Int, reader.spec().sample_format);

	let start = Instant::now();
	let c1: Vec<i16> = reader
		.into_samples()
		.step_by(2)
		.map(|s| s.unwrap())
		.collect();
	let elapsed = start.elapsed();
	println!("Load: {elapsed:.3?}");

	let start = Instant::now();
	let buffer = render(&c1, width, height);
	let elapsed = start.elapsed();
	println!("Render: {elapsed:.3?}");

	// gui stuff
	// in future, make buffer mut and update it in main loop
	let mut window = Window::new(
		"Test - ESC to exit",
		width,
		height,
		WindowOptions::default(),
	)
	.unwrap_or_else(|e| {
		panic!("{}", e);
	});
	// 16700 for 60 fps, 6900 for 144
	window.limit_update_rate(Some(std::time::Duration::from_micros(6800)));

	// main loop
	while window.is_open() && !window.is_key_down(Key::Escape) && !window.is_key_down(Key::Q) {
		window.update_with_buffer(&buffer, width, height).unwrap();
	}
}
