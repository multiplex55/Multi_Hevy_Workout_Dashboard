use phf::phf_map;
use serde::{Deserialize, Serialize};

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

/// Metadata about an exercise including its primary and secondary muscles.
#[derive(Debug, Clone, Copy)]
pub struct ExerciseInfo {
    pub primary: &'static str,
    pub secondary: &'static [&'static str],
    pub kind: ExerciseType,
}

pub static EXERCISES: phf::Map<&'static str, ExerciseInfo> = phf_map! {
    // Chest
    "Barbell Bench Press" => ExerciseInfo { primary: "Chest", secondary: &["Triceps", "Front Delts"], kind: ExerciseType::Compound },
    "Incline DB Press" => ExerciseInfo { primary: "Chest", secondary: &["Triceps", "Front Delts"], kind: ExerciseType::Compound },
    "Flat DB Press" => ExerciseInfo { primary: "Chest", secondary: &["Triceps", "Front Delts"], kind: ExerciseType::Compound },
    "Pec Deck" => ExerciseInfo { primary: "Chest", secondary: &["Front Delts"], kind: ExerciseType::Isolation },
    "Cable Crossover" => ExerciseInfo { primary: "Chest", secondary: &["Front Delts"], kind: ExerciseType::Isolation },
    "Push-Up" => ExerciseInfo { primary: "Chest", secondary: &["Triceps", "Core"], kind: ExerciseType::Compound },
    "Machine Chest Press" => ExerciseInfo { primary: "Chest", secondary: &["Triceps", "Front Delts"], kind: ExerciseType::Compound },
    "Incline Cable Press" => ExerciseInfo { primary: "Chest", secondary: &["Triceps", "Front Delts"], kind: ExerciseType::Compound },
    "Dips (Chest Lean)" => ExerciseInfo { primary: "Chest", secondary: &["Triceps", "Front Delts"], kind: ExerciseType::Compound },
    // Shoulders
    "Overhead Barbell Press" => ExerciseInfo { primary: "Shoulders", secondary: &["Triceps", "Upper Chest"], kind: ExerciseType::Compound },
    "Dumbbell Shoulder Press" => ExerciseInfo { primary: "Shoulders", secondary: &["Triceps", "Upper Chest"], kind: ExerciseType::Compound },
    "Arnold Press" => ExerciseInfo { primary: "Shoulders", secondary: &["Triceps", "Front Delts"], kind: ExerciseType::Compound },
    "Seated Lateral Raise" => ExerciseInfo { primary: "Side Delts", secondary: &["Upper Traps"], kind: ExerciseType::Isolation },
    "Cable Lateral Raise" => ExerciseInfo { primary: "Side Delts", secondary: &["Upper Traps"], kind: ExerciseType::Isolation },
    "Dumbbell Front Raise" => ExerciseInfo { primary: "Front Delts", secondary: &["Upper Chest"], kind: ExerciseType::Isolation },
    "Rear Delt Fly (Machine or DB)" => ExerciseInfo { primary: "Rear Delts", secondary: &["Upper Back", "Traps"], kind: ExerciseType::Isolation },
    "Face Pull" => ExerciseInfo { primary: "Rear Delts", secondary: &["Traps", "Rotator Cuff"], kind: ExerciseType::Isolation },
    "Upright Row" => ExerciseInfo { primary: "Traps", secondary: &["Side Delts"], kind: ExerciseType::Compound },
    "Landmine Press" => ExerciseInfo { primary: "Shoulders", secondary: &["Triceps", "Core"], kind: ExerciseType::Compound },
    "Bradford Press" => ExerciseInfo { primary: "Shoulders", secondary: &["Triceps", "Traps"], kind: ExerciseType::Compound },
    "Dumbbell Lying Rear Delt Raise" => ExerciseInfo { primary: "Rear Delts", secondary: &["Traps"], kind: ExerciseType::Isolation },
    "Cable Y-Raise" => ExerciseInfo { primary: "Shoulders", secondary: &["Traps", "Rear Delts"], kind: ExerciseType::Isolation },
    "Machine Overhead Press" => ExerciseInfo { primary: "Shoulders", secondary: &["Triceps"], kind: ExerciseType::Compound },
    "Wall Slide" => ExerciseInfo { primary: "Shoulders", secondary: &["Traps", "Rotator Cuff"], kind: ExerciseType::Isolation },
    "Band Pull Apart" => ExerciseInfo { primary: "Rear Delts", secondary: &["Upper Back"], kind: ExerciseType::Isolation },
    "Cable Rear Delt Row" => ExerciseInfo { primary: "Rear Delts", secondary: &["Traps"], kind: ExerciseType::Isolation },
    "Rope Face Pull to Neck" => ExerciseInfo { primary: "Rear Delts", secondary: &["Traps", "Rotator Cuff"], kind: ExerciseType::Isolation },
    "Battle Ropes" => ExerciseInfo { primary: "Shoulders", secondary: &["Core", "Biceps"], kind: ExerciseType::Compound },
    // Legs - Quads dominant
    "Barbell Back Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Hamstrings", "Core"], kind: ExerciseType::Compound },
    "Front Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Core"], kind: ExerciseType::Compound },
    "Goblet Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Core"], kind: ExerciseType::Compound },
    "Hack Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Hamstrings"], kind: ExerciseType::Compound },
    "Leg Press" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Hamstrings"], kind: ExerciseType::Compound },
    "Walking Lunges" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Hamstrings"], kind: ExerciseType::Compound },
    "Bulgarian Split Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Core"], kind: ExerciseType::Compound },
    "Step-Ups" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Hamstrings"], kind: ExerciseType::Compound },
    "Sissy Squat" => ExerciseInfo { primary: "Quads", secondary: &["Core"], kind: ExerciseType::Isolation },
    "Reverse Lunge" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Hamstrings"], kind: ExerciseType::Compound },
    "Curtsy Lunge" => ExerciseInfo { primary: "Glutes", secondary: &["Adductors", "Quads"], kind: ExerciseType::Compound },
    "Leg Extension" => ExerciseInfo { primary: "Quads", secondary: &[], kind: ExerciseType::Isolation },
    "Smith Machine Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Hamstrings"], kind: ExerciseType::Compound },
    "Kneeling Squat" => ExerciseInfo { primary: "Glutes", secondary: &["Quads"], kind: ExerciseType::Compound },
    "Isometric Wall Sit" => ExerciseInfo { primary: "Quads", secondary: &["Glutes"], kind: ExerciseType::Isometric },
    "Sled Push" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Hamstrings", "Calves"], kind: ExerciseType::Compound },
    // Posterior chain
    "Romanian Deadlift" => ExerciseInfo { primary: "Hamstrings", secondary: &["Glutes", "Lower Back"], kind: ExerciseType::Compound },
    "Conventional Deadlift" => ExerciseInfo { primary: "Back", secondary: &["Glutes", "Hamstrings", "Traps"], kind: ExerciseType::Compound },
    "Sumo Deadlift" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Hamstrings", "Adductors"], kind: ExerciseType::Compound },
    "Trap Bar Deadlift" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Traps", "Hamstrings"], kind: ExerciseType::Compound },
    "Good Morning" => ExerciseInfo { primary: "Hamstrings", secondary: &["Glutes", "Lower Back"], kind: ExerciseType::Compound },
    "Seated Leg Curl" => ExerciseInfo { primary: "Hamstrings", secondary: &["Calves"], kind: ExerciseType::Isolation },
    "Lying Leg Curl" => ExerciseInfo { primary: "Hamstrings", secondary: &["Calves"], kind: ExerciseType::Isolation },
    "Standing Leg Curl (Cable)" => ExerciseInfo { primary: "Hamstrings", secondary: &[], kind: ExerciseType::Isolation },
    "Nordic Curl" => ExerciseInfo { primary: "Hamstrings", secondary: &["Glutes"], kind: ExerciseType::Isolation },
    "Seated Good Morning" => ExerciseInfo { primary: "Hamstrings", secondary: &["Glutes", "Core"], kind: ExerciseType::Compound },
    // Calves & Tibialis
    "Standing Calf Raise" => ExerciseInfo { primary: "Calves", secondary: &[], kind: ExerciseType::Isolation },
    "Seated Calf Raise" => ExerciseInfo { primary: "Calves", secondary: &[], kind: ExerciseType::Isolation },
    "Tibialis Raise" => ExerciseInfo { primary: "Tibialis Anterior", secondary: &[], kind: ExerciseType::Isolation },
    // Back
    "Pull-Up / Chin-Up" => ExerciseInfo { primary: "Lats", secondary: &["Biceps", "Core"], kind: ExerciseType::Compound },
    "Lat Pulldown" => ExerciseInfo { primary: "Lats", secondary: &["Biceps", "Rear Delts"], kind: ExerciseType::Compound },
    "Barbell Row" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts", "Biceps"], kind: ExerciseType::Compound },
    "Dumbbell Row" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts", "Biceps"], kind: ExerciseType::Compound },
    "T-Bar Row" => ExerciseInfo { primary: "Lats", secondary: &["Biceps", "Rear Delts"], kind: ExerciseType::Compound },
    "Seated Cable Row" => ExerciseInfo { primary: "Lats", secondary: &["Biceps", "Rear Delts"], kind: ExerciseType::Compound },
    "Straight-Arm Lat Pulldown" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts"], kind: ExerciseType::Isolation },
    "Cable Pullover" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts"], kind: ExerciseType::Isolation },
    "Incline Prone Row (Chest Support)" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts", "Biceps"], kind: ExerciseType::Compound },
    "Meadows Row" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts", "Biceps"], kind: ExerciseType::Compound },
    "Incline Bench Row (DB)" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts", "Biceps"], kind: ExerciseType::Compound },
    "Kneeling Single-Arm Lat Pulldown" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts", "Biceps"], kind: ExerciseType::Compound },
    "Behind-the-Neck Pulldown" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts"], kind: ExerciseType::Compound },
    "Inverted Row (Bodyweight)" => ExerciseInfo { primary: "Lats", secondary: &["Biceps", "Rear Delts"], kind: ExerciseType::Compound },
    "Gironda Sternum Chin-Up" => ExerciseInfo { primary: "Lats", secondary: &["Rear Delts", "Core"], kind: ExerciseType::Compound },
    "TRX Row" => ExerciseInfo { primary: "Lats", secondary: &["Biceps", "Core"], kind: ExerciseType::Compound },
    "Single-Arm DB Row" => ExerciseInfo { primary: "Lats", secondary: &["Biceps", "Rear Delts"], kind: ExerciseType::Compound },
    "Rowing Machine" => ExerciseInfo { primary: "Lats", secondary: &["Quads", "Biceps"], kind: ExerciseType::Compound },
    "Shrugs" => ExerciseInfo { primary: "Traps", secondary: &[], kind: ExerciseType::Isolation },
    "Rack Pull" => ExerciseInfo { primary: "Traps", secondary: &["Glutes", "Hamstrings", "Back"], kind: ExerciseType::Compound },
    // Arms - Biceps
    "Barbell Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Forearms"], kind: ExerciseType::Isolation },
    "Dumbbell Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Forearms"], kind: ExerciseType::Isolation },
    "Preacher Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation },
    "Incline DB Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation },
    "Cable Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation },
    "Concentration Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation },
    "Hammer Curl" => ExerciseInfo { primary: "Biceps (Brachialis)", secondary: &["Forearms"], kind: ExerciseType::Isolation },
    "Zottman Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Forearms"], kind: ExerciseType::Isolation },
    "One-Arm Cable Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Forearms"], kind: ExerciseType::Isolation },
    "Machine Bicep Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation },
    "Drag Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation },
    "Incline Cable Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation },
    "Cross-Body Hammer Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Brachialis"], kind: ExerciseType::Isolation },
    "Cable Rope Hammer Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Forearms"], kind: ExerciseType::Isolation },
    "Cable Reverse Curl" => ExerciseInfo { primary: "Forearms", secondary: &["Biceps"], kind: ExerciseType::Isolation },
    "Wrist Roller" => ExerciseInfo { primary: "Forearms", secondary: &["Grip"], kind: ExerciseType::Isolation },
    "Spider Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation },
    "Reverse Curl" => ExerciseInfo { primary: "Forearms", secondary: &["Biceps (Brachialis)"], kind: ExerciseType::Isolation },
    "Wrist Curl / Reverse Wrist Curl" => ExerciseInfo { primary: "Forearms", secondary: &[], kind: ExerciseType::Isolation },
    // Arms - Triceps
    "Triceps Pushdown" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    "Overhead Triceps Extension" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    "Skull Crushers" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    "Dips (Triceps Focus)" => ExerciseInfo { primary: "Triceps", secondary: &["Chest", "Front Delts"], kind: ExerciseType::Compound },
    "Close-Grip Bench Press" => ExerciseInfo { primary: "Triceps", secondary: &["Chest", "Front Delts"], kind: ExerciseType::Compound },
    "Lying Triceps Extension" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    "Triceps Dip Machine" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    "Crossbody Cable Triceps Extension" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    "Triceps Rope Overhead Extension" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    "DB Tate Press" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    "Incline Skull Crusher" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    "Decline Close-Grip Bench" => ExerciseInfo { primary: "Triceps", secondary: &["Chest", "Front Delts"], kind: ExerciseType::Compound },
    "Cable Triceps Kickback" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    "V-Bar Pushdown" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    "Kickbacks (Cable/DB)" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    // Core
    "Crunches" => ExerciseInfo { primary: "Abs", secondary: &[], kind: ExerciseType::Isolation },
    "Hanging Leg Raise" => ExerciseInfo { primary: "Abs", secondary: &["Hip Flexors"], kind: ExerciseType::Isolation },
    "Cable Crunch" => ExerciseInfo { primary: "Abs", secondary: &[], kind: ExerciseType::Isolation },
    "Reverse Crunch" => ExerciseInfo { primary: "Lower Abs", secondary: &[], kind: ExerciseType::Isolation },
    "Russian Twist" => ExerciseInfo { primary: "Obliques", secondary: &["Core"], kind: ExerciseType::Isolation },
    "Side Plank" => ExerciseInfo { primary: "Obliques", secondary: &["Core"], kind: ExerciseType::Isolation },
    "Plank" => ExerciseInfo { primary: "Core", secondary: &["Glutes", "Abs"], kind: ExerciseType::Isometric },
    "Weighted Plank" => ExerciseInfo { primary: "Core", secondary: &["Abs", "Glutes"], kind: ExerciseType::Isometric },
    "Cable Woodchopper" => ExerciseInfo { primary: "Obliques", secondary: &["Core"], kind: ExerciseType::Isolation },
    "Dragon Flag" => ExerciseInfo { primary: "Abs", secondary: &["Hip Flexors"], kind: ExerciseType::Isolation },
    "Hanging Knee Raise" => ExerciseInfo { primary: "Abs", secondary: &["Hip Flexors"], kind: ExerciseType::Isolation },
    "Stability Ball Crunch" => ExerciseInfo { primary: "Abs", secondary: &[], kind: ExerciseType::Isolation },
    "Weighted Decline Sit-Up" => ExerciseInfo { primary: "Abs", secondary: &[], kind: ExerciseType::Isolation },
    "Toes-to-Bar" => ExerciseInfo { primary: "Abs", secondary: &["Core", "Lats"], kind: ExerciseType::Compound },
    "Ab Wheel Rollout" => ExerciseInfo { primary: "Abs", secondary: &["Core", "Lats"], kind: ExerciseType::Isolation },
    "Cable Side Bend" => ExerciseInfo { primary: "Obliques", secondary: &["Core"], kind: ExerciseType::Isolation },
    "Kneeling Cable Crunch" => ExerciseInfo { primary: "Abs", secondary: &[], kind: ExerciseType::Isolation },
    "Bear Crawl" => ExerciseInfo { primary: "Core", secondary: &["Shoulders", "Quads"], kind: ExerciseType::Compound },
    "Cable Pallof Press" => ExerciseInfo { primary: "Obliques", secondary: &["Core", "Shoulders"], kind: ExerciseType::Isometric },
    "Dead Bug" => ExerciseInfo { primary: "Core", secondary: &[], kind: ExerciseType::Isometric },
    "Bird Dog" => ExerciseInfo { primary: "Core", secondary: &["Glutes", "Lower Back"], kind: ExerciseType::Isometric },
    "Pallof Press" => ExerciseInfo { primary: "Obliques", secondary: &["Core", "Shoulders"], kind: ExerciseType::Isometric },
    "Landmine Rotation" => ExerciseInfo { primary: "Obliques", secondary: &["Core", "Shoulders"], kind: ExerciseType::Compound },
    // Misc / full body
    "Farmer's Carry" => ExerciseInfo { primary: "Traps", secondary: &["Core", "Forearms", "Grip"], kind: ExerciseType::Compound },
    "Hip Thrust" => ExerciseInfo { primary: "Glutes", secondary: &["Hamstrings", "Core"], kind: ExerciseType::Compound },
    "Glute Bridge" => ExerciseInfo { primary: "Glutes", secondary: &["Hamstrings"], kind: ExerciseType::Compound },
    "Glute Kickback (Cable)" => ExerciseInfo { primary: "Glutes", secondary: &["Hamstrings"], kind: ExerciseType::Isolation },
    "Cable Abduction" => ExerciseInfo { primary: "Glute Medius", secondary: &[], kind: ExerciseType::Isolation },
    "Adductor Machine" => ExerciseInfo { primary: "Adductors", secondary: &[], kind: ExerciseType::Isolation },
    "Abductor Machine" => ExerciseInfo { primary: "Glute Medius", secondary: &[], kind: ExerciseType::Isolation },
    "Donkey Kick" => ExerciseInfo { primary: "Glutes", secondary: &["Hamstrings"], kind: ExerciseType::Isolation },
    "Cable Kickback" => ExerciseInfo { primary: "Glutes", secondary: &["Hamstrings"], kind: ExerciseType::Isolation },
    "Machine Glute Kickback" => ExerciseInfo { primary: "Glutes", secondary: &["Hamstrings"], kind: ExerciseType::Isolation },
    "Standing Abduction (Band)" => ExerciseInfo { primary: "Glute Medius", secondary: &[], kind: ExerciseType::Isolation },
    "Standing Adduction (Cable)" => ExerciseInfo { primary: "Adductors", secondary: &[], kind: ExerciseType::Isolation },
    "Single-Leg Glute Bridge" => ExerciseInfo { primary: "Glutes", secondary: &["Hamstrings", "Core"], kind: ExerciseType::Isolation },
    "Fire Hydrant (Band or BW)" => ExerciseInfo { primary: "Glute Medius", secondary: &["Core"], kind: ExerciseType::Isolation },
    "Sled Drag" => ExerciseInfo { primary: "Full Body", secondary: &["Glutes", "Quads", "Hamstrings"], kind: ExerciseType::Compound },
    "Sled Row" => ExerciseInfo { primary: "Back", secondary: &["Biceps", "Core"], kind: ExerciseType::Compound },
    "Stepmill" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Calves"], kind: ExerciseType::Cardio },
    "VersaClimber" => ExerciseInfo { primary: "Full Body", secondary: &["Core", "Shoulders", "Legs"], kind: ExerciseType::Cardio },
    "Speed Skater (BW Plyo)" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Adductors"], kind: ExerciseType::Plyometric },
    "Broad Jump" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Calves"], kind: ExerciseType::Plyometric },
    "Power Clean" => ExerciseInfo { primary: "Full Body", secondary: &["Traps", "Glutes", "Quads", "Core"], kind: ExerciseType::Compound },
    "Clean and Jerk" => ExerciseInfo { primary: "Full Body", secondary: &["Triceps", "Quads", "Glutes"], kind: ExerciseType::Compound },
    "Snatch" => ExerciseInfo { primary: "Full Body", secondary: &["Traps", "Glutes", "Core", "Shoulders"], kind: ExerciseType::Compound },
    "Assault Bike" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Hamstrings"], kind: ExerciseType::Compound },
    "Neck Flexion (Harness)" => ExerciseInfo { primary: "Neck", secondary: &[], kind: ExerciseType::Isolation },
    "Neck Extension (Harness)" => ExerciseInfo { primary: "Neck", secondary: &[], kind: ExerciseType::Isolation },
    "Neck Curl (Weighted)" => ExerciseInfo { primary: "Neck", secondary: &[], kind: ExerciseType::Isolation },
    "Neck Extension (Plate)" => ExerciseInfo { primary: "Neck", secondary: &[], kind: ExerciseType::Isolation },
    "Band External Rotation" => ExerciseInfo { primary: "Rotator Cuff", secondary: &[], kind: ExerciseType::Isolation },
    "Cable L-Fly" => ExerciseInfo { primary: "Rotator Cuff", secondary: &[], kind: ExerciseType::Isolation },
    "Cuban Rotation" => ExerciseInfo { primary: "Rotator Cuff", secondary: &["Rear Delts"], kind: ExerciseType::Isolation },
    "Cable Internal Rotation" => ExerciseInfo { primary: "Rotator Cuff", secondary: &[], kind: ExerciseType::Isolation },
    "Cuban Press" => ExerciseInfo { primary: "Rotator Cuff", secondary: &["Shoulders", "Traps"], kind: ExerciseType::Isolation },
    "External Rotation (Cable)" => ExerciseInfo { primary: "Rotator Cuff", secondary: &[], kind: ExerciseType::Isolation },
    "Scapular Push-Up" => ExerciseInfo { primary: "Serratus Anterior", secondary: &["Chest", "Shoulders"], kind: ExerciseType::Isolation },
    // Legacy synonyms
    "Bench" => ExerciseInfo { primary: "Chest", secondary: &[], kind: ExerciseType::Compound },
    "Bench Press" => ExerciseInfo { primary: "Chest", secondary: &[], kind: ExerciseType::Compound },
    "Squat" => ExerciseInfo { primary: "Quads", secondary: &[], kind: ExerciseType::Compound },
    "Deadlift" => ExerciseInfo { primary: "Back", secondary: &[], kind: ExerciseType::Compound },
    "Lying Leg Curl (Machine)" => ExerciseInfo { primary: "Hamstrings", secondary: &["Calves"], kind: ExerciseType::Isolation },
    "Bicep Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Forearms"], kind: ExerciseType::Isolation },
};

/// Lookup full information for a given exercise name.
pub fn info_for(exercise: &str) -> Option<&'static ExerciseInfo> {
    EXERCISES.get(exercise)
}

/// Convenience wrapper returning only the primary muscle group.
pub fn body_part_for(exercise: &str) -> Option<&'static str> {
    info_for(exercise).map(|i| i.primary)
}

/// Return a sorted list of all unique primary muscle groups.
pub fn primary_muscle_groups() -> Vec<&'static str> {
    use std::collections::BTreeSet;
    let mut set = BTreeSet::new();
    for info in EXERCISES.values() {
        set.insert(info.primary);
    }
    set.into_iter().collect()
}
