# One Billion Row Challenge

A Rust implementation for the 1 Billion Row Challenge (1BRC) that generates a large dataset of temperature records and calculates statistics for each city.

## Setup

Note: The following instructions are for Linux and macOS. If you are using Windows, you can install the Windows Subsystem for Linux (WSL) or use a virtual machine.

1. Install Rust: https://www.rust-lang.org/tools/install
2. Clone the repository: `git clone git@github.com:ViniciosLugli/1brc.git`
3. Change to the project directory: `cd 1brc`
4. You can set `DATA_FILE_PATH` environment variable to specify the path where the generated dataset will be saved. If not set, the default path is the local folder.
5. Run the project: `cargo run --release`

## Solution Strategy

I started the project by reading the challenge description and understanding the requirements. I then broke down the problem into smaller tasks and identified the key components of the solution. I decided to implement the solution in Rust due to its performance, safety, and concurrency features.

### Data Processing

-   The data processing step involves reading the generated dataset, parsing the temperature data, and calculating statistics for each city.
-   The dataset is read in parallel to utilize multiple CPU cores effectively.
-   City-wise statistics such as minimum, maximum, and average temperatures are calculated and stored.
-   The results are displayed in a tabular format for easy interpretation.

### Implementation

-   The solution utilizes asynchronous programming with Tokio and Rayon to achieve concurrency and parallelism.
-   Tokio is used for asynchronous I/O operations, while Rayon is used for parallel data processing.
-   Memory-mapping is employed to efficiently read large files without loading them entirely into memory.
-   Atomic counters are used for tracking progress and coordinating tasks across multiple threads.
-   The solution is designed to scale with the available CPU cores to maximize performance.

### Performance

-   The solution is designed to utilize multi-core processors efficiently for both data generation and processing.
-   Memory-mapping is employed to minimize memory usage and optimize I/O performance.
-   Parallelization and concurrency are used to maximize throughput and reduce processing time.

## Known Issues

The generated dataset has a rand seed problem setting all the temperatures to the same value, i know that this is a problem, but this dont impact the final result of the project, so I decided to not fix it and focus on the main problem.

## Conclusion

The final result in my testing environment is a execution time of `27.51` seconds for the 1 billion rows generated dataset, which is a significant improvement over the initial implementation of project that took `~300` seconds. Far away from the `~6` seconds of the most optimized solution found in the internet, but I'm happy with the result, considering that is my first time with this kind of problem and all the implementation was done in a single day :)

## Helpers / References

-   https://github.com/danieljl/rust-1B-row-challenge
-   https://github.com/tumdum/1brc
