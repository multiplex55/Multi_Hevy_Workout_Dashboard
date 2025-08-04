use phf::phf_map;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};

use crate::{WorkoutEntry, exercise_mapping};

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

/// Metadata about an exercise including its primary and secondary muscles.
#[derive(Debug, Clone, Copy)]
pub struct ExerciseInfo {
    pub primary: &'static str,
    pub secondary: &'static [&'static str],
    pub kind: ExerciseType,
    pub difficulty: Option<Difficulty>,
    pub equipment: Option<Equipment>,
}

pub static EXERCISES: phf::Map<&'static str, ExerciseInfo> = phf_map! {
    // Chest
    "Barbell Bench Press" => ExerciseInfo { primary: "Chest", secondary: &["Triceps", "Front Delts"], kind: ExerciseType::Compound, difficulty: Some(Difficulty::Intermediate), equipment: Some(Equipment::Barbell) },
    "Incline DB Press" => ExerciseInfo { primary: "Chest", secondary: &["Triceps", "Front Delts"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Flat DB Press" => ExerciseInfo { primary: "Chest", secondary: &["Triceps", "Front Delts"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Pec Deck" => ExerciseInfo { primary: "Chest", secondary: &["Front Delts"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cable Crossover" => ExerciseInfo { primary: "Chest", secondary: &["Front Delts"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Push-Up" => ExerciseInfo { primary: "Chest", secondary: &["Triceps", "Core"], kind: ExerciseType::Compound, difficulty: Some(Difficulty::Beginner), equipment: Some(Equipment::Bodyweight) },
    "Machine Chest Press" => ExerciseInfo { primary: "Chest", secondary: &["Triceps", "Front Delts"], kind: ExerciseType::Compound, difficulty: Some(Difficulty::Beginner), equipment: Some(Equipment::Machine) },
    "Incline Cable Press" => ExerciseInfo { primary: "Chest", secondary: &["Triceps", "Front Delts"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Dips (Chest Lean)" => ExerciseInfo { primary: "Chest", secondary: &["Triceps", "Front Delts"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    // Shoulders
    "Overhead Barbell Press" => ExerciseInfo { primary: "Shoulders", secondary: &["Triceps", "Upper Chest"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Dumbbell Shoulder Press" => ExerciseInfo { primary: "Shoulders", secondary: &["Triceps", "Upper Chest"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Arnold Press" => ExerciseInfo { primary: "Shoulders", secondary: &["Triceps", "Front Delts"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Seated Lateral Raise" => ExerciseInfo { primary: "Side Delts", secondary: &["Upper Traps"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cable Lateral Raise" => ExerciseInfo { primary: "Side Delts", secondary: &["Upper Traps"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Dumbbell Front Raise" => ExerciseInfo { primary: "Front Delts", secondary: &["Upper Chest"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Rear Delt Fly (Machine or DB)" => ExerciseInfo { primary: "Rear Delts", secondary: &["Upper Back", "Traps"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Face Pull" => ExerciseInfo { primary: "Rear Delts", secondary: &["Traps", "Rotator Cuff"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Upright Row" => ExerciseInfo { primary: "Traps", secondary: &["Side Delts"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Landmine Press" => ExerciseInfo { primary: "Shoulders", secondary: &["Triceps", "Core"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Bradford Press" => ExerciseInfo { primary: "Shoulders", secondary: &["Triceps", "Traps"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Dumbbell Lying Rear Delt Raise" => ExerciseInfo { primary: "Rear Delts", secondary: &["Traps"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cable Y-Raise" => ExerciseInfo { primary: "Shoulders", secondary: &["Traps", "Rear Delts"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Machine Overhead Press" => ExerciseInfo { primary: "Shoulders", secondary: &["Triceps"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Wall Slide" => ExerciseInfo { primary: "Shoulders", secondary: &["Traps", "Rotator Cuff"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Band Pull Apart" => ExerciseInfo { primary: "Rear Delts", secondary: &["Upper Back"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cable Rear Delt Row" => ExerciseInfo { primary: "Rear Delts", secondary: &["Traps"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Rope Face Pull to Neck" => ExerciseInfo { primary: "Rear Delts", secondary: &["Traps", "Rotator Cuff"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Battle Ropes" => ExerciseInfo { primary: "Shoulders", secondary: &["Core", "Biceps"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    // Legs - Quads dominant
    "Barbell Back Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Hamstrings", "Core"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Front Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Core"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Goblet Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Core"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Hack Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Hamstrings"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Leg Press" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Hamstrings"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Walking Lunges" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Hamstrings"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Bulgarian Split Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Core"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Step-Ups" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Hamstrings"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Sissy Squat" => ExerciseInfo { primary: "Quads", secondary: &["Core"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Reverse Lunge" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Hamstrings"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Curtsy Lunge" => ExerciseInfo { primary: "Glutes", secondary: &["Adductors", "Quads"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Leg Extension" => ExerciseInfo { primary: "Quads", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Smith Machine Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Hamstrings"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Kneeling Squat" => ExerciseInfo { primary: "Glutes", secondary: &["Quads"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Isometric Wall Sit" => ExerciseInfo { primary: "Quads", secondary: &["Glutes"], kind: ExerciseType::Isometric, difficulty: None, equipment: None },
    "Sled Push" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Hamstrings", "Calves"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    // Posterior chain
    "Romanian Deadlift" => ExerciseInfo { primary: "Hamstrings", secondary: &["Glutes", "Lower Back"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Conventional Deadlift" => ExerciseInfo { primary: "Back", secondary: &["Glutes", "Hamstrings", "Traps"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Sumo Deadlift" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Hamstrings", "Adductors"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Trap Bar Deadlift" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Traps", "Hamstrings"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Good Morning" => ExerciseInfo { primary: "Hamstrings", secondary: &["Glutes", "Lower Back"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Seated Leg Curl" => ExerciseInfo { primary: "Hamstrings", secondary: &["Calves"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Lying Leg Curl" => ExerciseInfo { primary: "Hamstrings", secondary: &["Calves"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Standing Leg Curl (Cable)" => ExerciseInfo { primary: "Hamstrings", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Nordic Curl" => ExerciseInfo { primary: "Hamstrings", secondary: &["Glutes"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Seated Good Morning" => ExerciseInfo { primary: "Hamstrings", secondary: &["Glutes", "Core"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    // Calves & Tibialis
    "Standing Calf Raise" => ExerciseInfo { primary: "Calves", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Seated Calf Raise" => ExerciseInfo { primary: "Calves", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Tibialis Raise" => ExerciseInfo { primary: "Tibialis Anterior", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    // Back
    "Pull-Up / Chin-Up" => ExerciseInfo { primary: "Lats", secondary: &["Biceps", "Core"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Lat Pulldown" => ExerciseInfo { primary: "Lats", secondary: &["Biceps", "Rear Delts"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Barbell Row" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts", "Biceps"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Dumbbell Row" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts", "Biceps"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "T-Bar Row" => ExerciseInfo { primary: "Lats", secondary: &["Biceps", "Rear Delts"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Seated Cable Row" => ExerciseInfo { primary: "Lats", secondary: &["Biceps", "Rear Delts"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Straight-Arm Lat Pulldown" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cable Pullover" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Incline Prone Row (Chest Support)" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts", "Biceps"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Meadows Row" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts", "Biceps"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Incline Bench Row (DB)" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts", "Biceps"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Kneeling Single-Arm Lat Pulldown" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts", "Biceps"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Behind-the-Neck Pulldown" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Inverted Row (Bodyweight)" => ExerciseInfo { primary: "Lats", secondary: &["Biceps", "Rear Delts"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Gironda Sternum Chin-Up" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts", "Core"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "TRX Row" => ExerciseInfo { primary: "Lats", secondary: &["Biceps", "Core"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Single-Arm DB Row" => ExerciseInfo { primary: "Lats", secondary: &["Biceps", "Rear Delts"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Rowing Machine" => ExerciseInfo { primary: "Lats", secondary: &["Quads", "Biceps"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Shrugs" => ExerciseInfo { primary: "Traps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Rack Pull" => ExerciseInfo { primary: "Traps", secondary: &["Glutes", "Hamstrings", "Back"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    // Arms - Biceps
    "Barbell Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Forearms"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Dumbbell Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Forearms"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Preacher Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Incline DB Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cable Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Concentration Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Hammer Curl" => ExerciseInfo { primary: "Biceps (Brachialis)", secondary: &["Forearms"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Zottman Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Forearms"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "One-Arm Cable Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Forearms"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Machine Bicep Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Drag Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Incline Cable Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cross-Body Hammer Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Brachialis"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cable Rope Hammer Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Forearms"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cable Reverse Curl" => ExerciseInfo { primary: "Forearms", secondary: &["Biceps"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Wrist Roller" => ExerciseInfo { primary: "Forearms", secondary: &["Grip"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Spider Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Reverse Curl" => ExerciseInfo { primary: "Forearms", secondary: &["Biceps (Brachialis)"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Wrist Curl / Reverse Wrist Curl" => ExerciseInfo { primary: "Forearms", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    // Arms - Triceps
    "Triceps Pushdown" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Overhead Triceps Extension" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Skull Crushers" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Dips (Triceps Focus)" => ExerciseInfo { primary: "Triceps", secondary: &["Chest", "Front Delts"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Close-Grip Bench Press" => ExerciseInfo { primary: "Triceps", secondary: &["Chest", "Front Delts"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Lying Triceps Extension" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Triceps Dip Machine" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Crossbody Cable Triceps Extension" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Triceps Rope Overhead Extension" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "DB Tate Press" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Incline Skull Crusher" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Decline Close-Grip Bench" => ExerciseInfo { primary: "Triceps", secondary: &["Chest", "Front Delts"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Cable Triceps Kickback" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "V-Bar Pushdown" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Kickbacks (Cable/DB)" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    // Core
    "Crunches" => ExerciseInfo { primary: "Abs", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Hanging Leg Raise" => ExerciseInfo { primary: "Abs", secondary: &["Hip Flexors"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cable Crunch" => ExerciseInfo { primary: "Abs", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Reverse Crunch" => ExerciseInfo { primary: "Lower Abs", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Russian Twist" => ExerciseInfo { primary: "Obliques", secondary: &["Core"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Side Plank" => ExerciseInfo { primary: "Obliques", secondary: &["Core"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Plank" => ExerciseInfo { primary: "Core", secondary: &["Glutes", "Abs"], kind: ExerciseType::Isometric, difficulty: None, equipment: None },
    "Weighted Plank" => ExerciseInfo { primary: "Core", secondary: &["Abs", "Glutes"], kind: ExerciseType::Isometric, difficulty: None, equipment: None },
    "Cable Woodchopper" => ExerciseInfo { primary: "Obliques", secondary: &["Core"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Dragon Flag" => ExerciseInfo { primary: "Abs", secondary: &["Hip Flexors"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Hanging Knee Raise" => ExerciseInfo { primary: "Abs", secondary: &["Hip Flexors"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Stability Ball Crunch" => ExerciseInfo { primary: "Abs", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Weighted Decline Sit-Up" => ExerciseInfo { primary: "Abs", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Toes-to-Bar" => ExerciseInfo { primary: "Abs", secondary: &["Core", "Lats"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Ab Wheel Rollout" => ExerciseInfo { primary: "Abs", secondary: &["Core", "Lats"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cable Side Bend" => ExerciseInfo { primary: "Obliques", secondary: &["Core"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Kneeling Cable Crunch" => ExerciseInfo { primary: "Abs", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Bear Crawl" => ExerciseInfo { primary: "Core", secondary: &["Shoulders", "Quads"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Cable Pallof Press" => ExerciseInfo { primary: "Obliques", secondary: &["Core", "Shoulders"], kind: ExerciseType::Isometric, difficulty: None, equipment: None },
    "Dead Bug" => ExerciseInfo { primary: "Core", secondary: &[], kind: ExerciseType::Isometric, difficulty: None, equipment: None },
    "Bird Dog" => ExerciseInfo { primary: "Core", secondary: &["Glutes", "Lower Back"], kind: ExerciseType::Isometric, difficulty: None, equipment: None },
    "Pallof Press" => ExerciseInfo { primary: "Obliques", secondary: &["Core", "Shoulders"], kind: ExerciseType::Isometric, difficulty: None, equipment: None },
    "Landmine Rotation" => ExerciseInfo { primary: "Obliques", secondary: &["Core", "Shoulders"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    // Misc / full body
    "Farmer's Carry" => ExerciseInfo { primary: "Traps", secondary: &["Core", "Forearms", "Grip"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Hip Thrust" => ExerciseInfo { primary: "Glutes", secondary: &["Hamstrings", "Core"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Glute Bridge" => ExerciseInfo { primary: "Glutes", secondary: &["Hamstrings"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Glute Kickback (Cable)" => ExerciseInfo { primary: "Glutes", secondary: &["Hamstrings"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cable Abduction" => ExerciseInfo { primary: "Glute Medius", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Adductor Machine" => ExerciseInfo { primary: "Adductors", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Abductor Machine" => ExerciseInfo { primary: "Glute Medius", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Donkey Kick" => ExerciseInfo { primary: "Glutes", secondary: &["Hamstrings"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cable Kickback" => ExerciseInfo { primary: "Glutes", secondary: &["Hamstrings"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Machine Glute Kickback" => ExerciseInfo { primary: "Glutes", secondary: &["Hamstrings"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Standing Abduction (Band)" => ExerciseInfo { primary: "Glute Medius", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Standing Adduction (Cable)" => ExerciseInfo { primary: "Adductors", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Single-Leg Glute Bridge" => ExerciseInfo { primary: "Glutes", secondary: &["Hamstrings", "Core"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Fire Hydrant (Band or BW)" => ExerciseInfo { primary: "Glute Medius", secondary: &["Core"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Sled Drag" => ExerciseInfo { primary: "Full Body", secondary: &["Glutes", "Quads", "Hamstrings"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Sled Row" => ExerciseInfo { primary: "Back", secondary: &["Biceps", "Core"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Stepmill" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Calves"], kind: ExerciseType::Cardio, difficulty: None, equipment: None },
    "VersaClimber" => ExerciseInfo { primary: "Full Body", secondary: &["Core", "Shoulders", "Legs"], kind: ExerciseType::Cardio, difficulty: None, equipment: None },
    "Speed Skater (BW Plyo)" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Adductors"], kind: ExerciseType::Plyometric, difficulty: None, equipment: None },
    "Broad Jump" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Calves"], kind: ExerciseType::Plyometric, difficulty: None, equipment: None },
    "Power Clean" => ExerciseInfo { primary: "Full Body", secondary: &["Traps", "Glutes", "Quads", "Core"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Clean and Jerk" => ExerciseInfo { primary: "Full Body", secondary: &["Triceps", "Quads", "Glutes"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Snatch" => ExerciseInfo { primary: "Full Body", secondary: &["Traps", "Glutes", "Core", "Shoulders"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Assault Bike" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Hamstrings"], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Neck Flexion (Harness)" => ExerciseInfo { primary: "Neck", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Neck Extension (Harness)" => ExerciseInfo { primary: "Neck", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Neck Curl (Weighted)" => ExerciseInfo { primary: "Neck", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Neck Extension (Plate)" => ExerciseInfo { primary: "Neck", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Band External Rotation" => ExerciseInfo { primary: "Rotator Cuff", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cable L-Fly" => ExerciseInfo { primary: "Rotator Cuff", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cuban Rotation" => ExerciseInfo { primary: "Rotator Cuff", secondary: &["Rear Delts"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cable Internal Rotation" => ExerciseInfo { primary: "Rotator Cuff", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Cuban Press" => ExerciseInfo { primary: "Rotator Cuff", secondary: &["Shoulders", "Traps"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "External Rotation (Cable)" => ExerciseInfo { primary: "Rotator Cuff", secondary: &[], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    "Scapular Push-Up" => ExerciseInfo { primary: "Serratus Anterior", secondary: &["Chest", "Shoulders"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
    // Legacy synonyms
    "Bench" => ExerciseInfo { primary: "Chest", secondary: &[], kind: ExerciseType::Compound, difficulty: Some(Difficulty::Intermediate), equipment: Some(Equipment::Barbell) },
    "Bench Press" => ExerciseInfo { primary: "Chest", secondary: &[], kind: ExerciseType::Compound, difficulty: Some(Difficulty::Intermediate), equipment: Some(Equipment::Barbell) },
    "Squat" => ExerciseInfo { primary: "Quads", secondary: &[], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Deadlift" => ExerciseInfo { primary: "Back", secondary: &[], kind: ExerciseType::Compound, difficulty: None, equipment: None },
    "Lying Leg Curl (Machine)" => ExerciseInfo { primary: "Hamstrings", secondary: &["Calves"], kind: ExerciseType::Isolation, difficulty: Some(Difficulty::Beginner), equipment: Some(Equipment::Machine) },
    "Bicep Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Forearms"], kind: ExerciseType::Isolation, difficulty: None, equipment: None },
};

/// Lookup full information for a given exercise name.
pub fn info_for(exercise: &str) -> Option<&'static ExerciseInfo> {
    EXERCISES.get(exercise)
}

/// Convenience wrapper returning only the primary muscle group.
pub fn body_part_for(exercise: &str) -> Option<String> {
    if let Some(m) = exercise_mapping::get(exercise) {
        if !m.primary.is_empty() {
            return Some(m.primary);
        }
    }
    info_for(exercise).map(|i| i.primary.to_string())
}

/// Return a sorted list of all unique primary muscle groups.
pub fn primary_muscle_groups() -> Vec<String> {
    let mut set = BTreeSet::new();
    for info in EXERCISES.values() {
        set.insert(info.primary.to_string());
    }
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

/// Convenience wrapper returning the equipment classification.
pub fn equipment_for(exercise: &str) -> Option<Equipment> {
    info_for(exercise).and_then(|i| i.equipment)
}

/// Return the high level category for an exercise from custom mappings.
pub fn category_for(exercise: &str) -> Option<String> {
    exercise_mapping::get(exercise).map(|m| m.category)
}

/// Iterate over the provided workouts and ensure the muscle mapping for each
/// exercise reflects the latest built-in defaults.
///
/// For every unique exercise in `workouts` this will look up the static
/// [`ExerciseInfo`] and overwrite the primary and secondary muscle groups in the
/// persistent [`exercise_mapping`] store. The existing category is preserved.
///
/// The function returns the number of exercises that were updated. Callers are
/// responsible for persisting changes via [`exercise_mapping::save`].
pub fn update_mappings_from_workouts(workouts: &[WorkoutEntry]) -> usize {
    let mut updated = 0;
    let mut seen: HashSet<String> = HashSet::new();
    for w in workouts {
        if !seen.insert(w.exercise.clone()) {
            continue;
        }
        if let Some(info) = info_for(&w.exercise) {
            let mut entry = exercise_mapping::get(&w.exercise).unwrap_or_default();
            let primary = info.primary.to_string();
            let secondary: Vec<String> = info.secondary.iter().map(|s| s.to_string()).collect();
            if entry.primary != primary || entry.secondary != secondary {
                entry.primary = primary;
                entry.secondary = secondary;
                exercise_mapping::set(w.exercise.clone(), entry);
                updated += 1;
            }
        }
    }
    updated
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RawWorkoutRow;

    #[test]
    fn updates_mappings() {
        let workout = WorkoutEntry {
            date: "2024-01-01".into(),
            exercise: "Barbell Bench Press".into(),
            weight: Some(100.0),
            reps: Some(5),
            raw: RawWorkoutRow::default(),
        };
        // Ensure clean state
        exercise_mapping::remove(&workout.exercise);
        let updated = update_mappings_from_workouts(&[workout]);
        assert_eq!(updated, 1);
        let mapping = exercise_mapping::get("Barbell Bench Press").unwrap();
        assert_eq!(mapping.primary, "Chest");
        assert!(mapping.secondary.contains(&"Triceps".to_string()));
        exercise_mapping::remove("Barbell Bench Press");
    }
}
