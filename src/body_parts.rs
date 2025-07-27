use phf::phf_map;

pub static EXERCISE_BODY_PARTS: phf::Map<&'static str, &'static str> = phf_map! {
    "Bench" => "Chest",
    "Bench Press" => "Chest",
    "Squat" => "Legs",
    "Deadlift" => "Back",
    "Lying Leg Curl (Machine)" => "Legs",
    "Bicep Curl" => "Arms",
};

pub fn body_part_for(exercise: &str) -> Option<&'static str> {
    EXERCISE_BODY_PARTS.get(exercise).copied()
}
