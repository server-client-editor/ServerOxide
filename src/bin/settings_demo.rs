use ServerOxide::settings::*;

fn main() {
    // Load settings from the default location
    let project_settings = parse_settings(None).unwrap();
    println!("Loaded settings: {:?}", project_settings);

    // Attempt to load from an invalid path (expected to fail)
    let is_err = parse_settings(Some("")).is_err();
    println!("Error on invalid path: {:?}", is_err);
}