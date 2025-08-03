use crate::{
    WorkoutEntry,
    analysis::{BasicStats, ExerciseRecord, ExerciseStats},
};
use serde::Serialize;
use std::io::Write;
use std::path::Path;

pub fn write_json<T: Serialize + ?Sized, P: AsRef<Path>>(
    value: &T,
    path: P,
) -> std::io::Result<()> {
    let file = std::fs::File::create(path)?;
    serde_json::to_writer_pretty(file, value)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

pub fn write_csv<T: Serialize>(writer: impl Write, records: &[T]) -> csv::Result<()> {
    let mut wtr = csv::Writer::from_writer(writer);
    for r in records {
        wtr.serialize(r)?;
    }
    wtr.flush().map_err(Into::into)
}

pub fn save_basic_stats_csv<P: AsRef<Path>>(path: P, stats: &BasicStats) -> csv::Result<()> {
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.serialize(stats)?;
    wtr.flush().map_err(Into::into)
}

pub fn save_basic_stats_json<P: AsRef<Path>>(path: P, stats: &BasicStats) -> std::io::Result<()> {
    write_json(stats, path)
}

pub fn save_exercise_stats_csv<P: AsRef<Path>>(
    path: P,
    stats: &[(String, ExerciseStats)],
) -> csv::Result<()> {
    #[derive(Serialize)]
    struct Row<'a> {
        exercise: &'a str,
        #[serde(flatten)]
        stats: &'a ExerciseStats,
    }
    let mut rows = Vec::new();
    for (ex, s) in stats {
        rows.push(Row {
            exercise: ex,
            stats: s,
        });
    }
    write_csv(std::fs::File::create(path)?, &rows)
}

pub fn save_exercise_stats_json<P: AsRef<Path>>(
    path: P,
    stats: &[(String, ExerciseStats)],
) -> std::io::Result<()> {
    write_json(stats, path)
}

pub fn save_entries_csv<P: AsRef<Path>>(path: P, entries: &[WorkoutEntry]) -> csv::Result<()> {
    let mut wtr = csv::Writer::from_path(path)?;
    for e in entries {
        wtr.serialize(e)?;
    }
    wtr.flush().map_err(Into::into)
}

pub fn save_entries_json<P: AsRef<Path>>(path: P, entries: &[WorkoutEntry]) -> std::io::Result<()> {
    write_json(entries, path)
}

pub fn save_prs_csv<P: AsRef<Path>>(
    path: P,
    records: &[(String, ExerciseRecord)],
) -> csv::Result<()> {
    #[derive(Serialize)]
    struct Row<'a> {
        exercise: &'a str,
        max_weight: Option<f32>,
        max_volume: Option<f32>,
        best_est_1rm: Option<f32>,
    }
    let rows: Vec<Row> = records
        .iter()
        .map(|(ex, r)| Row {
            exercise: ex,
            max_weight: r.max_weight,
            max_volume: r.max_volume,
            best_est_1rm: r.best_est_1rm,
        })
        .collect();
    write_csv(std::fs::File::create(path)?, &rows)
}

pub fn save_prs_json<P: AsRef<Path>>(
    path: P,
    records: &[(String, ExerciseRecord)],
) -> std::io::Result<()> {
    write_json(records, path)
}

#[derive(Serialize)]
pub struct StatsExport<'a> {
    pub summary: &'a BasicStats,
    pub exercises: &'a [(String, ExerciseStats)],
}

pub fn save_stats_json<P: AsRef<Path>>(
    path: P,
    summary: &BasicStats,
    exercises: &[(String, ExerciseStats)],
) -> std::io::Result<()> {
    let export = StatsExport { summary, exercises };
    write_json(&export, path)
}

pub fn save_stats_csv<P: AsRef<Path>>(
    path: P,
    summary: &BasicStats,
    exercises: &[(String, ExerciseStats)],
) -> csv::Result<()> {
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.serialize(summary)?;
    #[derive(Serialize)]
    struct Row<'a> {
        exercise: &'a str,
        #[serde(flatten)]
        stats: &'a ExerciseStats,
    }
    for (ex, s) in exercises {
        wtr.serialize(Row {
            exercise: ex,
            stats: s,
        })?;
    }
    wtr.flush().map_err(Into::into)
}
