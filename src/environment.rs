use std::{any::type_name, env, fmt::Display, str::FromStr};

pub fn get_environment_variable_or<T: Display + FromStr>(name: &str, default: T) -> T {
    let variable_text = match env::var(name) {
        Ok(v) => v,
        Err(_) => {
            println!(
                "Environment variable '{}' not defined. Using default value of '{}'.",
                name, default
            );
            default.to_string()
        }
    };

    match variable_text.parse::<T>() {
        Ok(v) => v,
        Err(_) => {
            println!(
                "Could not convert value '{}' of '{}' to {}. Using default value of '{}'",
                variable_text,
                name,
                type_name::<T>(),
                default
            );
            default
        }
    }
}
