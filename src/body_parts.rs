use phf::phf_map;

/// Type of exercise based on muscle engagement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExerciseType {
    Compound,
    Isolation,
    Isometric,
}

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
    // Legs - Quads dominant
    "Barbell Back Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Hamstrings", "Core"], kind: ExerciseType::Compound },
    "Front Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Core"], kind: ExerciseType::Compound },
    "Goblet Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Core"], kind: ExerciseType::Compound },
    "Hack Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Hamstrings"], kind: ExerciseType::Compound },
    "Leg Press" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Hamstrings"], kind: ExerciseType::Compound },
    "Walking Lunges" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Hamstrings"], kind: ExerciseType::Compound },
    "Bulgarian Split Squat" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Core"], kind: ExerciseType::Compound },
    "Step-Ups" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Hamstrings"], kind: ExerciseType::Compound },
    "Sled Push" => ExerciseInfo { primary: "Quads", secondary: &["Glutes", "Hamstrings", "Calves"], kind: ExerciseType::Compound },
    // Posterior chain
    "Romanian Deadlift" => ExerciseInfo { primary: "Hamstrings", secondary: &["Glutes", "Lower Back"], kind: ExerciseType::Compound },
    "Conventional Deadlift" => ExerciseInfo { primary: "Back", secondary: &["Glutes", "Hamstrings", "Traps"], kind: ExerciseType::Compound },
    "Sumo Deadlift" => ExerciseInfo { primary: "Glutes", secondary: &["Quads", "Hamstrings", "Adductors"], kind: ExerciseType::Compound },
    "Good Morning" => ExerciseInfo { primary: "Hamstrings", secondary: &["Glutes", "Lower Back"], kind: ExerciseType::Compound },
    "Seated Leg Curl" => ExerciseInfo { primary: "Hamstrings", secondary: &["Calves"], kind: ExerciseType::Isolation },
    "Lying Leg Curl" => ExerciseInfo { primary: "Hamstrings", secondary: &["Calves"], kind: ExerciseType::Isolation },
    "Standing Leg Curl (Cable)" => ExerciseInfo { primary: "Hamstrings", secondary: &[], kind: ExerciseType::Isolation },
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
    "Shrugs" => ExerciseInfo { primary: "Traps", secondary: &[], kind: ExerciseType::Isolation },
    // Arms - Biceps
    "Barbell Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Forearms"], kind: ExerciseType::Isolation },
    "Dumbbell Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Forearms"], kind: ExerciseType::Isolation },
    "Preacher Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation },
    "Incline DB Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation },
    "Cable Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation },
    "Concentration Curl" => ExerciseInfo { primary: "Biceps", secondary: &[], kind: ExerciseType::Isolation },
    "Hammer Curl" => ExerciseInfo { primary: "Biceps (Brachialis)", secondary: &["Forearms"], kind: ExerciseType::Isolation },
    "Zottman Curl" => ExerciseInfo { primary: "Biceps", secondary: &["Forearms"], kind: ExerciseType::Isolation },
    "Reverse Curl" => ExerciseInfo { primary: "Forearms", secondary: &["Biceps (Brachialis)"], kind: ExerciseType::Isolation },
    "Wrist Curl / Reverse Wrist Curl" => ExerciseInfo { primary: "Forearms", secondary: &[], kind: ExerciseType::Isolation },
    // Arms - Triceps
    "Triceps Pushdown" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    "Overhead Triceps Extension" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    "Skull Crushers" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    "Dips (Triceps Focus)" => ExerciseInfo { primary: "Triceps", secondary: &["Chest", "Front Delts"], kind: ExerciseType::Compound },
    "Close-Grip Bench Press" => ExerciseInfo { primary: "Triceps", secondary: &["Chest", "Front Delts"], kind: ExerciseType::Compound },
    "Lying Triceps Extension" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    "Kickbacks (Cable/DB)" => ExerciseInfo { primary: "Triceps", secondary: &[], kind: ExerciseType::Isolation },
    // Core
    "Crunches" => ExerciseInfo { primary: "Abs", secondary: &[], kind: ExerciseType::Isolation },
    "Hanging Leg Raise" => ExerciseInfo { primary: "Abs", secondary: &["Hip Flexors"], kind: ExerciseType::Isolation },
    "Cable Crunch" => ExerciseInfo { primary: "Abs", secondary: &[], kind: ExerciseType::Isolation },
    "Reverse Crunch" => ExerciseInfo { primary: "Lower Abs", secondary: &[], kind: ExerciseType::Isolation },
    "Russian Twist" => ExerciseInfo { primary: "Obliques", secondary: &["Core"], kind: ExerciseType::Isolation },
    "Side Plank" => ExerciseInfo { primary: "Obliques", secondary: &["Core"], kind: ExerciseType::Isolation },
    "Plank" => ExerciseInfo { primary: "Core", secondary: &["Glutes", "Abs"], kind: ExerciseType::Isometric },
    "Cable Woodchopper" => ExerciseInfo { primary: "Obliques", secondary: &["Core"], kind: ExerciseType::Isolation },
    "Dragon Flag" => ExerciseInfo { primary: "Abs", secondary: &["Hip Flexors"], kind: ExerciseType::Isolation },
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
