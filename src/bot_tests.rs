#[allow(unused_imports)]
use super::*;
use nostr_sdk::{EventBuilder, Keys};
use std::time::{Duration, Instant};

#[test]
#[cfg(not(debug_assertions))]
fn test_pow_event_generation_duration() {
	dotenv().ok();
	let keys = Keys::generate();
	let test_msg = "test message";
	let pow_difficulty: u8 = env::var("POW_DIFFICULTY").unwrap().parse().unwrap();
	let iterations = 3;

	let mut total_duration = Duration::new(0, 0);
	for _ in 0..iterations {
		let start = Instant::now();
		let _ = EventBuilder::text_note(test_msg, None)
			.to_pow_event(&keys, pow_difficulty)
			.unwrap();
		let duration = start.elapsed();
		total_duration += duration;
	}

	let average_duration = total_duration / iterations as u32;
	println!(
		"Average duration of PoW {pow_difficulty} over {} iterations: {:?}",
		iterations, average_duration
	);
	assert!(average_duration != Duration::new(0, 0));
}
