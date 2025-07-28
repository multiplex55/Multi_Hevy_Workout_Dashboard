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

Hevy exports workouts as a CSV. The dashboard expects the extended export which
contains columns like the workout start time and exercise information. A snippet
looks like:

```csv
"title","start_time","end_time","description","exercise_title","superset_id","exercise_notes","set_index","set_type","weight_lbs","reps","distance_miles","duration_seconds","rpe"
"Week 12 - Lower - Strength","26 Jul 2025, 07:06","26 Jul 2025, 08:11","...","Lying Leg Curl (Machine)",,"",0,"warmup",100,10,,,
```

Each row represents a single set. All columns are parsed and stored:
`title`, `start_time`, `end_time`, `description`, `exercise_title`, `superset_id`,
`exercise_notes`, `set_index`, `set_type`, `weight_lbs`, `reps`,
`distance_miles`, `duration_seconds` and `rpe`. The graphs are generated from
the workout date, exercise name, weight and reps.

## Features

* **File Selection** – Click **Load CSV** to choose a Hevy export file. The app parses the file and stores the workout entries.
* **Drag & Drop** – You can also drop a `.csv` file onto the window to load it directly.
* **Stat Calculations** – After loading, the program computes totals such as average sets per workout, average reps per set, days between sessions and most frequent exercise.
* **Plots** – For the selected exercise you can view:
  * Weight over time
  * Estimated one‑rep max over time
  * Sets per day (bar chart)
* **Compare Window** – Open from the exercise menu to view each selected exercise in its own plot column for side‑by‑side comparison.
* **Raw Entry Table** – Open *Raw Entries* from the **File** menu to see every
  workout set in a sortable table. Columns can be sorted and the table respects
  the date range from the settings as well as an exercise filter.

Use the drop‑down at the top of the window to change the exercise displayed in the plots. Open the **Settings** window from the **File** menu to choose whether each plot is shown and select the formula (Epley or Brzycki) used for estimating 1RM.
