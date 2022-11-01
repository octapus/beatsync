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

fn read_file(reader: WavReader<BufReader<File>>) -> Option<(Vec<i16>, Vec<i16>)> {
	let mut c1 = Vec::with_capacity((reader.len() / 2).try_into().ok()?);
	let mut c2 = Vec::with_capacity((reader.len() / 2).try_into().ok()?);
	let mut iter = reader.into_samples();
	assert_eq!(iter.len() % 2, 0);
	loop {
		match (iter.next(), iter.next()) {
			(Some(a), Some(b)) => {
				c1.push(a.ok()?);
				c2.push(b.ok()?);
			}
			_ => return Some((c1, c2)),
		};
	}
}

fn render(input: &[i16], output: &mut [u32], width: usize) {
	assert_eq!(output.len() % width, 0);
	let height = output.len() / width;
	let max_list = input
		.chunks_exact(input.len() / width)
		.map(|chunk| chunk.iter().fold(0, |acc, &x| std::cmp::max(acc, x.abs())));
	assert_eq!(max_list.len(), width);
	for (i, max) in max_list.enumerate() {
		let scaled_height =
			usize::try_from(max).unwrap() * height / usize::try_from(i16::MAX).unwrap();
		let diff = (height - scaled_height) / 2;
		for j in diff..(height - diff) {
			output[j * width + i] = u32::MAX;
		}
	}
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
	let (c1, c2) = read_file(reader).unwrap();
	let elapsed = start.elapsed();
	println!("Load: {elapsed:.3?}");

	let start = Instant::now();
	let mut buffer = vec![0u32; width * height];
	render(&c1, &mut buffer[0..(width * height / 2)], width);
	render(
		&c2,
		&mut buffer[(width * height / 2)..(width * height)],
		width,
	);
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
