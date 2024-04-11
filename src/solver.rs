use async_std::io::{self};

use bstr::{BString, ByteSlice};
use indicatif::{ProgressBar, ProgressStyle};
use memmap::MmapOptions;
use prettytable::{Cell, Row, Table};
use rustc_hash::FxHashMap as HashMap;
use std::{
	sync::{
		atomic::{AtomicUsize, Ordering},
		Arc,
	},
	thread, usize,
};
use tokio::runtime::Builder as RuntimeBuilder;

// Define a struct CityStats to hold statistics for a city
#[derive(Debug, Default)]
struct CityStats {
	min: f64,
	max: f64,
	count: u64,
	sum: f64,
}

impl CityStats {
	// Method to update statistics with a new value
	fn update(&mut self, next_value: f64) {
		self.min = self.min.min(next_value);
		self.max = self.max.max(next_value);
		self.count += 1;
		self.sum += next_value;
	}

	// Method to merge statistics with another CityStats instance
	fn merge(&mut self, other: &CityStats) {
		self.min = self.min.min(other.min);
		self.max = self.max.max(other.max);
		self.count += other.count;
		self.sum += other.sum;
	}
}

// Define a struct DataProcessor to process data
struct DataProcessor {
	file_path: String, // Path to the file to be processed
}

impl DataProcessor {
	// Constructor method
	fn new(path: &str) -> Self {
		DataProcessor { file_path: path.to_owned() }
	}

	// Asynchronous method to process data
	async fn process(&self) -> io::Result<()> {
		const NUM_ROWS: usize = 1_000_000_000; // Number of rows
		let start_time = std::time::Instant::now(); // Start time
		let file = std::fs::File::open(&self.file_path)?; // Open file
		let mmap = unsafe { MmapOptions::new().map(&file) }?; // Memory map the file

		let global_lines_processed = Arc::new(AtomicUsize::new(0)); // Atomic counter for global lines processed
		let pb = Arc::new(ProgressBar::new_spinner()); // Progress bar

		let cores: usize = thread::available_parallelism()?.get() as usize; // Number of available CPU cores
		println!("🚀 Running on {} cores", cores); // Print the number of cores

		let chunk_size = mmap.len() / cores; // Calculate chunk size
		let mut chunks: Vec<(usize, usize)> = Vec::with_capacity(cores); // Vector to store chunks

		let mut start = 0;
		for _ in 0..cores {
			let end = (start + chunk_size).min(mmap.len());
			let next_new_line = memchr::memchr(b'\n', &mmap[end..]).unwrap_or(mmap.len() - end) + end;
			chunks.push((start, next_new_line));
			start = next_new_line + 1;
		}

		println!("📂 Reading data"); // Print status

		// Set progress bar style
		pb.set_style(
			ProgressStyle::default_spinner()
				.template("{spinner:.green} [{elapsed_precise}]({eta})  [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
				.unwrap()
				.progress_chars("#>-"),
		);
		pb.set_length(NUM_ROWS as u64); // Set progress bar length

		let mmap_arc = Arc::new(mmap); // Arc reference to mmap
		let global_lines_processed_clone = Arc::clone(&global_lines_processed);
		let pb_clone = Arc::clone(&pb);
		let mut tasks = vec![];

		// Spawn tasks for each chunk
		for (s, e) in chunks {
			let mmap_clone = Arc::clone(&mmap_arc);
			let global_lines_processed = Arc::clone(&global_lines_processed_clone);
			let pb = Arc::clone(&pb_clone);
			let task = tokio::spawn(async move { solve_for_part((s, e), &mmap_clone, global_lines_processed, pb).await });
			tasks.push(task);
		}

		let mut state: HashMap<BString, CityStats> = HashMap::default(); // HashMap to store city statistics

		// Await completion of tasks
		for task in tasks {
			let result_map = task.await?; // Await task completion and get result
			for (city, stats) in result_map.iter() {
				state.entry(city.clone()).or_insert(CityStats::default()).merge(stats);
			}
		}

		println!("📝 Writing results"); // Print status
		let all: Vec<_> = state.into_iter().collect(); // Convert state to a vector

		let mut table = Table::new(); // Create a new table
		table.add_row(row!["City", "Count", "Min", "Max", "Avg"]); // Add header row to table

		// Add data to the table
		for (city, stats) in all {
			let avg = stats.sum / stats.count as f64;
			table.add_row(Row::new(vec![
				Cell::new(city.to_str().unwrap()),
				Cell::new(&stats.count.to_string()),
				Cell::new(&format!("{:.2}", stats.min)),
				Cell::new(&format!("{:.2}", stats.max)),
				Cell::new(&format!("{:.2}", avg)),
			]));
		}

		table.printstd(); // Print the table
		println!("⏱️ Elapsed time: {:.2?}", start_time.elapsed()); // Print elapsed time

		Ok(()) // Return Ok result
	}
}

// Asynchronous function to solve for a part of the data
async fn solve_for_part(
	(start, end): (usize, usize),
	mem: &[u8],
	global_lines_processed: Arc<AtomicUsize>,
	pb: Arc<ProgressBar>,
) -> HashMap<BString, CityStats> {
	let mut station = ProcessingStation::new(&mem[start..end], global_lines_processed, pb); // Create a ProcessingStation
	station.process().await; // Process data
	station.results // Return results
}

// Asynchronous function to read data
async fn read_data(data: &[u8], global_lines_processed: Arc<AtomicUsize>, pb: Arc<ProgressBar>) -> HashMap<BString, CityStats> {
	let mut map = HashMap::with_capacity_and_hasher(1024, Default::default()); // Create a HashMap
	let mut start = 0;
	let mut end = 0;
	let data_len = data.len();

	// Loop to read data
	while end < data_len {
		// Loop until end of data
		while end < data_len && data[end] != b'\n' {
			// Loop until end of line
			end += 1;
		}

		if let Some(semi_colon_index) = memchr::memchr(b';', &data[start..end]) {
			// Check for semi-colon
			let city = &data[start..start + semi_colon_index]; // Get city name
			let temp = &data[start + semi_colon_index + 1..end]; // Get temperature

			if let Ok(temp_val) = fast_float::parse::<f64, _>(temp) {
				// Parse temperature
				let city_stats = map.entry(BString::from(city)).or_insert_with(CityStats::default); // Insert city stats
				city_stats.update(temp_val); // Update city stats
			}
		}

		global_lines_processed.fetch_add(1, Ordering::SeqCst); // Increment global line counter
		pb.inc(1); // Increment progress bar

		// Update start and end indices for next line
		start = end + 1;
		end = start;
	}

	map // Return HashMap
}

// Define a struct ProcessingStation
struct ProcessingStation<'a> {
	data: &'a [u8],
	results: HashMap<BString, CityStats>,
	global_lines_processed: Arc<AtomicUsize>,
	pb: Arc<ProgressBar>,
}

impl<'a> ProcessingStation<'a> {
	// Constructor method
	fn new(data: &'a [u8], global_lines_processed: Arc<AtomicUsize>, pb: Arc<ProgressBar>) -> Self {
		ProcessingStation { data, results: HashMap::default(), global_lines_processed, pb }
	}

	// Asynchronous method to process data
	async fn process(&mut self) {
		self.results = read_data(self.data, Arc::clone(&self.global_lines_processed), Arc::clone(&self.pb)).await;
	}
}

// Function to solve
pub fn solve(path: &str) -> io::Result<()> {
	let processor = DataProcessor::new(&format!("{}/input.txt", path)); // Create a DataProcessor instance
	let cores: usize = thread::available_parallelism()?.get() as usize; // Get number of available CPU cores

	// Create a multi-threaded runtime with specified number of worker threads
	let runtime = RuntimeBuilder::new_multi_thread().worker_threads(cores).build().unwrap();

	// Execute the asynchronous process
	runtime.block_on(async {
		if let Err(e) = processor.process().await {
			eprintln!("Error: {}", e);
		}
	});

	Ok(()) // Return Ok result
}
