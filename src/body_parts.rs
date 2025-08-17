use phf::phf_map;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::exercise_mapping;

/// Type of exercise based on muscle engagement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExerciseType {
    Compound,
    Isolation,
    Isometric,
    Cardio,
    Plyometric,
}

pub const ALL_EXERCISE_TYPES: [ExerciseType; 5] = [
    ExerciseType::Compound,
    ExerciseType::Isolation,
    ExerciseType::Isometric,
    ExerciseType::Cardio,
    ExerciseType::Plyometric,
];

/// General difficulty level for an exercise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

pub const ALL_DIFFICULTIES: [Difficulty; 3] = [
    Difficulty::Beginner,
    Difficulty::Intermediate,
    Difficulty::Advanced,
];

/// Typical equipment used for an exercise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Equipment {
    Barbell,
    Dumbbell,
    Machine,
    Cable,
    Bodyweight,
    Other,
}

pub const ALL_EQUIPMENT: [Equipment; 6] = [
    Equipment::Barbell,
    Equipment::Dumbbell,
    Equipment::Machine,
    Equipment::Cable,
    Equipment::Bodyweight,
    Equipment::Other,
];

/// Metadata about an exercise excluding muscle mappings.
#[derive(Debug, Clone, Copy)]
pub struct ExerciseInfo {
    pub kind: ExerciseType,
    pub difficulty: Option<Difficulty>,
    pub equipment: Option<Equipment>,
}

pub static EXERCISES: phf::Map<&'static str, ExerciseInfo> = phf_map! {
    "Bench" => ExerciseInfo {
        kind: ExerciseType::Compound,
        difficulty: Some(Difficulty::Intermediate),
        equipment: Some(Equipment::Barbell),
    },
    "Squat" => ExerciseInfo {
        kind: ExerciseType::Compound,
        difficulty: None,
        equipment: None,
    },
    "Deadlift" => ExerciseInfo {
        kind: ExerciseType::Compound,
        difficulty: None,
        equipment: None,
    },
    "Push-Up" => ExerciseInfo {
        kind: ExerciseType::Compound,
        difficulty: Some(Difficulty::Beginner),
        equipment: Some(Equipment::Bodyweight),
    },
    "Lying Leg Curl (Machine)" => ExerciseInfo {
        kind: ExerciseType::Isolation,
        difficulty: Some(Difficulty::Beginner),
        equipment: Some(Equipment::Machine),
    },
};

/// Lookup full information for a given exercise name.
pub fn info_for(exercise: &str) -> Option<&'static ExerciseInfo> {
    EXERCISES.get(exercise)
}

/// Convenience wrapper returning only the primary muscle group from the
/// persisted mapping.
pub fn body_part_for(exercise: &str) -> Option<String> {
    exercise_mapping::get(exercise).and_then(|m| {
        if m.primary.is_empty() {
            None
        } else {
            Some(m.primary)
        }
    })
}

/// Return a sorted list of all unique primary muscle groups.
pub fn primary_muscle_groups() -> Vec<String> {
    let mut set = BTreeSet::new();
    for m in exercise_mapping::all().values() {
        if !m.primary.is_empty() {
            set.insert(m.primary.clone());
        }
    }
    set.into_iter().collect()
}

/// Convenience wrapper returning the difficulty classification.
pub fn difficulty_for(exercise: &str) -> Option<Difficulty> {
    info_for(exercise).and_then(|i| i.difficulty)
}

/// Convenience wrapper returning the typical equipment.
pub fn equipment_for(exercise: &str) -> Option<Equipment> {
    info_for(exercise).and_then(|i| i.equipment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_part_for_uses_mapping() {
        exercise_mapping::set(
            "Custom".into(),
            exercise_mapping::MuscleMapping {
                primary: "Back".into(),
                secondary: vec![],
                category: String::new(),
            },
        );
        assert_eq!(body_part_for("Custom"), Some("Back".into()));
        exercise_mapping::remove("Custom");
    }

    #[test]
    fn primary_groups_from_mappings() {
        exercise_mapping::set(
            "E1".into(),
            exercise_mapping::MuscleMapping {
                primary: "Chest".into(),
                secondary: vec![],
                category: String::new(),
            },
        );
        exercise_mapping::set(
            "E2".into(),
            exercise_mapping::MuscleMapping {
                primary: "Legs".into(),
                secondary: vec![],
                category: String::new(),
            },
        );
        let groups = primary_muscle_groups();
        assert!(groups.contains(&"Chest".to_string()));
        assert!(groups.contains(&"Legs".to_string()));
        exercise_mapping::remove("E1");
        exercise_mapping::remove("E2");
    }
}
