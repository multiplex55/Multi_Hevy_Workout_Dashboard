use crate::{
    analysis::{aggregate_weekly_summary, BasicStats, ExerciseRecord},
    WeightUnit, WorkoutEntry,
};
use maud::{html, Markup};
use plotters::prelude::*;
use std::path::Path;

trait FormatOption {
    fn fmt_opt(self) -> String;
}

impl FormatOption for Option<f32> {
    fn fmt_opt(self) -> String {
        self.map(|v| format!("{:.1}", v))
            .unwrap_or_else(|| "-".into())
    }
}

impl FormatOption for f32 {
    fn fmt_opt(self) -> String {
        format!("{:.1}", self)
    }
}

pub fn export_html_report<P: AsRef<Path>>(
    path: P,
    entries: &[WorkoutEntry],
    stats: &BasicStats,
    prs: &[(String, ExerciseRecord)],
    unit: WeightUnit,
) -> std::io::Result<()> {
    let path = path.as_ref();
    let chart_path = path.with_extension("png");
    let chart_file = match generate_volume_chart(entries, unit, &chart_path) {
        Ok(_) => chart_path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("")),
        Err(e) => {
            eprintln!("Failed to generate chart: {}", e);
            std::ffi::OsStr::new("")
        }
    };
    let markup = build_html(stats, prs, chart_file);
    std::fs::write(path, markup.into_string())
}

fn generate_volume_chart(
    entries: &[WorkoutEntry],
    unit: WeightUnit,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let weeks = aggregate_weekly_summary(entries, None, None);
    let root = BitMapBackend::new(path, (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;
    if weeks.is_empty() {
        root.present()?;
        return Ok(());
    }
    let max = weeks
        .iter()
        .map(|w| w.total_volume * unit.factor())
        .fold(0.0_f32, f32::max);
    let mut chart = ChartBuilder::on(&root)
        .caption("Weekly Volume", ("sans-serif", 25))
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0..weeks.len(), 0f32..max)?;
    chart
        .configure_mesh()
        .disable_mesh()
        .x_desc("Week")
        .y_desc(format!("Volume ({:?})", unit))
        .draw()?;
    chart.draw_series(LineSeries::new(
        weeks
            .iter()
            .enumerate()
            .map(|(i, w)| (i, w.total_volume * unit.factor())),
        &BLUE,
    ))?;
    root.present()?;
    Ok(())
}

fn build_html(
    stats: &BasicStats,
    prs: &[(String, ExerciseRecord)],
    chart_file: &std::ffi::OsStr,
) -> Markup {
    html! {
        html {
            head { meta charset="utf-8"; title { "Workout Report" } }
            body {
                h1 { "Summary" }
                table border="1" {
                    tr { th { "Total Workouts" } td { (stats.total_workouts) } }
                    tr { th { "Avg Sets/Workout" } td { (stats.avg_sets_per_workout.fmt_opt()) } }
                    tr { th { "Avg Reps/Set" } td { (stats.avg_reps_per_set.fmt_opt()) } }
                    tr { th { "Avg Days Between" } td { (stats.avg_days_between.fmt_opt()) } }
                    tr { th { "Most Common Exercise" } td { (stats.most_common_exercise.clone().unwrap_or_default()) } }
                }
                h1 { "Personal Records" }
                table border="1" {
                    tr { th { "Exercise" } th { "Max Weight" } th { "Max Volume" } th { "Best Est 1RM" } }
                    @for (ex, rec) in prs {
                        tr {
                            td { (ex) }
                            td { (rec.max_weight.map(|v| format!("{:.1}", v)).unwrap_or_else(|| "-".into())) }
                            td { (rec.max_volume.map(|v| format!("{:.1}", v)).unwrap_or_else(|| "-".into())) }
                            td { (rec.best_est_1rm.map(|v| format!("{:.1}", v)).unwrap_or_else(|| "-".into())) }
                        }
                    }
                }
                h1 { "Weekly Volume" }
                @if chart_file.is_empty() {
                    p { "Chart unavailable" }
                } @else {
                    img src=(chart_file.to_string_lossy());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn format_option_for_option_f32() {
        let none: Option<f32> = None;
        assert_eq!(none.fmt_opt(), "-");
        assert_eq!(Some(3.46_f32).fmt_opt(), "3.5");
        assert_eq!(Some(-1.27_f32).fmt_opt(), "-1.3");
        assert_eq!(Some(f32::INFINITY).fmt_opt(), "inf");
        assert_eq!(Some(f32::NEG_INFINITY).fmt_opt(), "-inf");
        assert_eq!(Some(f32::NAN).fmt_opt(), "NaN");
    }

    #[test]
    fn format_option_for_f32() {
        assert_eq!(3.46_f32.fmt_opt(), "3.5");
        assert_eq!((-1.27_f32).fmt_opt(), "-1.3");
        assert_eq!(f32::INFINITY.fmt_opt(), "inf");
        assert_eq!(f32::NEG_INFINITY.fmt_opt(), "-inf");
        assert_eq!(f32::NAN.fmt_opt(), "NaN");
    }

    #[test]
    fn build_html_renders_placeholders() {
        use crate::analysis::{BasicStats, ExerciseRecord};

        let stats = BasicStats {
            total_workouts: 10,
            avg_sets_per_workout: 3.46,
            avg_reps_per_set: 8.88,
            avg_days_between: 2.0,
            most_common_exercise: None,
        };

        let mut rec = ExerciseRecord::default();
        rec.max_weight = Some(150.0);
        rec.max_volume = None;
        rec.best_est_1rm = Some(200.0);
        let prs = vec![("Bench".to_string(), rec)];

        let output = build_html(&stats, &prs, OsStr::new("chart.png")).into_string();

        assert!(output.contains("3.5"));
        assert!(output.contains("8.9"));
        assert!(output.contains("2.0"));
        assert!(output.contains("<td>-</td>"));
        assert!(output.contains("150.0"));
        assert!(output.contains("200.0"));
    }

    #[test]
    fn build_html_handles_empty_chart_file() {
        use crate::analysis::{BasicStats, ExerciseRecord};

        let stats = BasicStats {
            total_workouts: 0,
            avg_sets_per_workout: 0.0,
            avg_reps_per_set: 0.0,
            avg_days_between: 0.0,
            most_common_exercise: None,
        };

        let prs: Vec<(String, ExerciseRecord)> = Vec::new();

        let output = build_html(&stats, &prs, OsStr::new(""));
        let output = output.into_string();

        assert!(output.contains("Chart unavailable"));
        assert!(!output.contains("<img"));
    }
}
