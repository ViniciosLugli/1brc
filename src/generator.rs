use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use rayon::prelude::*;
use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt, BufWriter};

// Asynchronous function to generate and write data to a file
pub async fn generate_and_write_data(path: &str) -> io::Result<()> {
	// Constants for controlling data generation
	const NUM_ROWS: usize = 1_000_000_000;
	const CHUNK_SIZE: usize = 1_000_000;
	const SUB_CHUNK_SIZE: usize = 100_000;
	const BUFFERED_BATCH_SIZE: usize = 10_000_000;

	// Create the output file
	let file = File::create(format!("{}/input.txt", path)).await?;
	let buffered_writer = BufWriter::new(file); // Wrap the file writer in a buffered writer
	let (tx, mut rx) = tokio::sync::mpsc::channel(500); // Create a channel for sending data between tasks

	// Set up progress bar
	let pb = ProgressBar::new(NUM_ROWS as u64);
	pb.set_style(
		ProgressStyle::default_bar()
			.template("{spinner:.green} [{elapsed_precise}]({eta}) [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg} lines")
			.unwrap()
			.progress_chars("#>-"),
	);

	// List of cities for data generation
	let cities = vec![
		"Hamburg",
		"Bulawayo",
		"Palembang",
		"St. John's",
		"Cracow",
		"Oslo",
		"Paris",
		"Tokyo",
		"New York",
		"Sydney",
		"Moscow",
		"Cape Town",
		"Buenos Aires",
		"Shanghai",
		"Mumbai",
		"Cairo",
		"London",
		"Nicola",
		"Rolandinho",
	];

	// Atomic variable to track the last generated temperature
	let last_temperature = Arc::new(AtomicUsize::new(0));

	// Spawn a Tokio task to generate data
	tokio::spawn(async move {
		for _ in 0..(NUM_ROWS / CHUNK_SIZE) {
			for _ in 0..(CHUNK_SIZE / SUB_CHUNK_SIZE) {
				let sub_chunk: Vec<String> = (0..SUB_CHUNK_SIZE)
					.into_par_iter() // Create parallel iterator
					.map(|_| {
						let city = cities[rand::thread_rng().gen_range(0..cities.len())]; // Randomly select a city
						let temperature = last_temperature.fetch_add(1, Ordering::SeqCst) as f64; // Get and increment temperature
						let temperature = (temperature % 90.0) as f64 - 20.0; // Generate temperature within range
						format!("{};{:.1}\n", city, temperature) // Format data as CSV line
					})
					.collect();

				tx.send(sub_chunk).await.expect("Failed to send data"); // Send data chunk through channel
				pb.inc(SUB_CHUNK_SIZE as u64); // Increment progress bar
			}
		}
		pb.finish_with_message("📦 Data generation complete"); // Finish progress bar when done
	});

	let mut writer = buffered_writer;
	let mut batch = Vec::with_capacity(BUFFERED_BATCH_SIZE);

	// Receive data from channel and write to file
	while let Some(data_chunk) = rx.recv().await {
		for data in data_chunk {
			batch.extend_from_slice(data.as_bytes());
			if batch.len() >= BUFFERED_BATCH_SIZE {
				writer.write_all(&batch).await?; // Write buffered data to file
				batch.clear();
			}
		}
	}

	if !batch.is_empty() {
		writer.write_all(&batch).await?; // Write remaining buffered data to file
	}
	writer.flush().await?; // Flush any remaining data to file

	Ok(()) // Return Ok() if successful
}

// Module containing utility functions for data generation
pub mod prelude {
	use tokio::runtime::Runtime;

	// Function to run data generation task
	pub fn run(path: &str) {
		println!("🚀 Launching data generation task...");
		block_on(super::generate_and_write_data(path)).expect("Failed to generate data");
		println!("✨ All data generated and written to file successfully.");
	}

	// Function to check if data file exists
	pub fn check_data(path: &str) -> bool {
		std::path::Path::new(&format!("{}/input.txt", path)).exists()
	}

	// Utility function to block on a future
	fn block_on<F: std::future::Future>(future: F) -> F::Output {
		let rt = Runtime::new().unwrap();
		rt.block_on(future)
	}
}
