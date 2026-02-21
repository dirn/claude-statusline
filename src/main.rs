use std::fmt;
use std::io;

use ansi_term::Color;
use serde::Deserialize;

// Colors
const ORANGE: u8 = 208;

// Icons
const MODEL_ICON: &str = "🤖";

#[derive(Deserialize)]
struct ClaudeStatusLineData {
    model: Model,
}

impl fmt::Display for ClaudeStatusLineData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let model = &self.model;
        write!(f, "{model}")
    }
}

#[derive(Deserialize)]
struct Model {
    display_name: String,
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_name = Color::Fixed(ORANGE).paint(&self.display_name);
        write!(f, "{MODEL_ICON} {display_name}")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data: ClaudeStatusLineData = serde_json::from_reader(io::stdin())?;
    println!("{data}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_output() {
        let data = ClaudeStatusLineData {
            model: Model {
                display_name: "Model Display Name".to_string(),
            },
        };

        let output = format!("{data}");
        assert!(output.contains("Model Display Name"));
    }

    #[test]
    fn json_deserialization() {
        let json = r#"{
            "model": {"display_name": "Sonnet 4.5"}
        }"#;

        let data: ClaudeStatusLineData = serde_json::from_str(json).unwrap();
        assert_eq!("Sonnet 4.5", data.model.display_name);
    }

    #[test]
    fn model_output() {
        let data = ClaudeStatusLineData {
            model: Model {
                display_name: "Model Display Name".to_string(),
            },
        };

        let output = format!("{data}");
        assert!(output.contains(MODEL_ICON));
        assert!(output.contains("Model Display Name"));
    }
}
