use std::{env, ffi::OsString, fs::File, io::BufReader, time::Instant};

use hound::WavReader;
use minifb::{Key, Window, WindowOptions};

const USAGE: &str = "Usage: beatsync <file.wav> [window width] [window height]";

const DEFAULT_WIDTH: usize = 1920;
const DEFAULT_HEIGHT: usize = 1080;

const ZOOM_DELTA1: f32 = 0.01;
const ZOOM_DELTA2: f32 = 0.0001;

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

fn get_chunks<T>(input: &[T], count: usize) -> Vec<&[T]> {
	let mut output = Vec::with_capacity(count);
	for i in 0..count {
		output.push(&input[(i * input.len() / count)..((i + 1) * input.len() / count)]);
	}
	assert_eq!(output.len(), count);
	output
}

fn render_channel(input: &[&[i16]], output: &mut [u32]) {
	let width = input.len();
	let height = output.len() / input.len();
	assert_eq!(output.len() % width, 0);
	let max_list = input
		.iter()
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

fn render(buffer: &mut [u32], c1: &[i16], c2: &[i16], width: usize, height: usize) {
	buffer.fill(0u32);
	let (cview1, cview2) = (get_chunks(c1, width), get_chunks(c2, width));
	render_channel(&cview1, &mut buffer[0..(width * height / 2)]);
	render_channel(&cview2, &mut buffer[(width * height / 2)..(width * height)]);
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

	let mut window = Window::new(
		"Beatsync - ESC or q to exit",
		width,
		height,
		WindowOptions::default(),
	)
	.expect("Failed to create window");
	// 16700 for 60 fps, 6900 for 144
	window.limit_update_rate(Some(std::time::Duration::from_micros(6800)));
	window.set_position(0, 0);

	let start = Instant::now();
	let (c1, c2) = read_file(reader).unwrap();
	assert_eq!(c1.len(), c2.len());
	let elapsed = start.elapsed();
	println!("Load: {elapsed:.3?}");

	let start = Instant::now();
	let mut buffer = vec![0u32; width * height];
	render(&mut buffer, &c1, &c2, width, height);
	let elapsed = start.elapsed();
	println!("Render: {elapsed:.3?}");

	// main loop
	let (mut render_start, mut render_len) = (0usize, c1.len());
	while window.is_open() && !window.is_key_down(Key::Escape) && !window.is_key_down(Key::Q) {
		if let Some((_dx, dy)) = window.get_scroll_wheel() {
			let delta = if window.is_key_down(Key::LeftShift) {
				ZOOM_DELTA2
			} else {
				ZOOM_DELTA1
			};
			if dy != 0.0 {
				let mut delta = (dy.abs() * delta * (c1.len() as f32)) as usize;
				if dy.is_sign_positive() && 2 * delta > render_len {
					delta = render_len / 2;
				};
				if dy.is_sign_negative() && 2 * delta + render_len > c1.len() {
					delta = (c1.len() - render_len) / 2;
				}
				if delta != 0 {
					if dy.is_sign_positive() {
						render_len -= 2 * delta;
						render_start += delta;
					} else {
						render_len += 2 * delta;
						render_start -= delta;
					}
				}
			}
			render(
				&mut buffer,
				&c1[render_start..(render_start + render_len)],
				&c2[render_start..(render_start + render_len)],
				width,
				height,
			);
		}
		window.update_with_buffer(&buffer, width, height).unwrap();
	}
}
