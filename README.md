# Multi Hevy Workout Dashboard

This project is a simple desktop tool written in Rust using [`eframe`](https://crates.io/crates/eframe) and `egui`. It loads a CSV export from the Hevy workout tracking app and visualizes your training data.

## Prerequisites

Install a recent Rust toolchain via [rustup](https://rustup.rs). Both `cargo` and `rustc` are required. The code was tested with Rust 1.87 but newer versions should also work.

## Building and Running

Clone the repository and build the project using `cargo`:

```bash
cargo build --release
```

Run the application with:

```bash
cargo run --release
```

On launch you will see a window with a **Load CSV** button.

## Hevy CSV Example

Hevy exports workouts as a CSV where each row contains the workout date, exercise name, weight and reps. Example:

```csv
date,exercise,weight,reps
2024-01-01,Squat,100,5
2024-01-01,Bench,80,5
2024-01-03,Squat,105,5
```

## Features

* **File Selection** – Click **Load CSV** to choose a Hevy export file. The app parses the file and stores the workout entries.
* **Stat Calculations** – After loading, the program computes totals such as average sets per workout, average reps per set, days between sessions and most frequent exercise.
* **Plots** – For the selected exercise you can view:
  * Weight over time
  * Estimated one‑rep max over time (Epley formula)
  * Sets per day (bar chart)

Use the drop‑down at the top of the window to change the exercise displayed in the plots.
