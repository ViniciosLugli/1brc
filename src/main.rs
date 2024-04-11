mod generator;
mod solver;
#[macro_use]
extern crate prettytable;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
	dotenvy::dotenv()?; // Load environment variables from .env file

	let path = std::env::var("DATA_FILE_PATH").unwrap_or_else(|_| ".".to_string());
	println!("📁 Data file path: {}", path);

	println!("🔍 Checking if data is already generated...");
	if generator::prelude::check_data(&path) {
		println!("✅ Data already generated!");
	} else {
		println!("🔄 Generating data...");
		generator::prelude::run(&path);
		println!("✅ Data generated successfully!");
	}

	println!("🧮 Processing data...");
	solver::solve(&path)?;
	println!("✅ Data processed successfully!");
	Ok(())
}
