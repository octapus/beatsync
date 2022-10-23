use std::env;

const USAGE: &str = "Usage: beatsync <file.wav>";

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
}
