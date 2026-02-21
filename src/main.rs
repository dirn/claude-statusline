use std::fmt;
use std::io;

use ansi_term::Color;
use serde::Deserialize;

// Colors
const BRIGHT_GREEN: u8 = 46;
const BRIGHT_YELLOW: u8 = 226;
const MAGENTA_PINK: u8 = 213;
const ORANGE: u8 = 208;
const PINK_RED: u8 = 203;

// Icons
const CONTEXT_ICON: &str = "🧠";
const MODEL_ICON: &str = "🤖";
const TOKENS_ICON: &str = "🪙";

const CONTEXT_BAR_WIDTH: usize = 10;
const CONTEXT_THRESHOLD_HIGH: i32 = 80; // Auto-compaction seems to kick in around 83%.
const CONTEXT_THRESHOLD_MEDIUM: i32 = 70;

#[derive(Deserialize)]
struct ClaudeStatusLineData {
    context_window: Option<ContextWindow>,
    model: Model,
}

impl fmt::Display for ClaudeStatusLineData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let model = &self.model;
        let percentage = self.context_window.clone().unwrap_or_default().percentage();
        let tokens = self.context_window.clone().unwrap_or_default().tokens();

        write!(f, "{model} | {percentage} | {tokens}")
    }
}

#[derive(Deserialize, Default, Clone)]
struct ContextWindow {
    #[serde(flatten)]
    percentage: Percentage,
    #[serde(flatten)]
    tokens: Tokens,
}

impl ContextWindow {
    fn percentage(&self) -> Percentage {
        self.percentage
    }

    fn tokens(&self) -> Tokens {
        self.tokens
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

#[derive(Deserialize, Default, Clone, Copy)]
struct Percentage {
    used_percentage: Option<f64>,
}

impl fmt::Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let percent = self.used_percentage.unwrap_or_default() as i32;
        let filled = percent * CONTEXT_BAR_WIDTH as i32 / 100;
        let bar = "▓".repeat(filled as usize) + &"░".repeat(CONTEXT_BAR_WIDTH - filled as usize);
        let context = self.color().paint(format!("{bar} {percent}%"));

        write!(f, "{CONTEXT_ICON} {context}")
    }
}

impl Percentage {
    fn color(&self) -> Color {
        let percent = self.used_percentage.unwrap_or_default() as i32;
        if percent > CONTEXT_THRESHOLD_HIGH {
            Color::Fixed(PINK_RED)
        } else if percent > CONTEXT_THRESHOLD_MEDIUM {
            Color::Fixed(BRIGHT_YELLOW)
        } else {
            Color::Fixed(BRIGHT_GREEN)
        }
    }
}

#[derive(Deserialize, Default, Clone, Copy)]
struct Tokens {
    total_input_tokens: Option<u64>,
    total_output_tokens: Option<u64>,
}

impl fmt::Display for Tokens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let input = self.total_input_tokens.unwrap_or_default();
        let output = self.total_output_tokens.unwrap_or_default();
        let tokens = Color::Fixed(MAGENTA_PINK).paint(format!("{input}↑ {output}↓"));

        write!(f, "{TOKENS_ICON} {tokens}")
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
    fn default_output() {
        let data = ClaudeStatusLineData {
            context_window: None,
            // This field is required.
            model: Model {
                display_name: "Model Display Name".to_string(),
            },
        };

        let output = format!("{data}");
        assert!(output.contains(" 0%"));
    }

    #[test]
    fn full_output() {
        let data = ClaudeStatusLineData {
            context_window: Some(ContextWindow {
                percentage: Percentage {
                    used_percentage: Some(20.0),
                },
                tokens: Tokens {
                    total_input_tokens: Some(5),
                    total_output_tokens: Some(10),
                },
            }),
            model: Model {
                display_name: "Model Display Name".to_string(),
            },
        };

        let output = format!("{data}");
        assert!(output.contains(MODEL_ICON));
        assert!(output.contains("Model Display Name"));
        assert!(output.contains(CONTEXT_ICON));
        assert!(output.contains("20%"));
        assert!(output.contains(TOKENS_ICON));
        assert!(output.contains("5↑"));
        assert!(output.contains("10↓"));
    }

    #[test]
    fn json_deserialization() {
        let json = r#"{
            "context_window": {"total_input_tokens": 10, "total_output_tokens": 5, "used_percentage": 1.0},
            "model": {"display_name": "Sonnet 4.5"}
        }"#;

        let data: ClaudeStatusLineData = serde_json::from_str(json).unwrap();
        assert_eq!("Sonnet 4.5", data.model.display_name);
    }

    #[test]
    fn model_output() {
        let model = Model {
            display_name: "Model Display Name".to_string(),
        };

        let output = format!("{model}");
        assert!(output.contains(MODEL_ICON));
        assert!(output.contains("Model Display Name"));
    }

    #[test]
    fn percentage_bar_empty() {
        let percentage = Percentage {
            used_percentage: Some(0.0),
        };

        let output = format!("{percentage}");
        assert!(output.contains("░"));
        assert!(!output.contains("▓"));
    }

    #[test]
    fn percentage_bar_full() {
        let percentage = Percentage {
            used_percentage: Some(100.0),
        };

        let output = format!("{percentage}");
        assert!(output.contains("▓"));
        assert!(!output.contains("░"));
    }

    #[test]
    fn percentage_bar_half_and_half() {
        let percentage = Percentage {
            used_percentage: Some(50.0),
        };

        let output = format!("{percentage}");
        // This could be more exact, but it's probably good enough.
        assert!(output.contains("▓"));
        assert!(output.contains("░"));
    }

    #[test]
    fn percentage_color_high() {
        let percentage = Percentage {
            used_percentage: Some(CONTEXT_THRESHOLD_HIGH as f64 + 1.0),
        };

        assert_eq!(Color::Fixed(PINK_RED), percentage.color());
    }

    #[test]
    fn percentage_color_low() {
        let percentage = Percentage {
            used_percentage: Some(CONTEXT_THRESHOLD_MEDIUM as f64),
        };

        assert_eq!(Color::Fixed(BRIGHT_GREEN), percentage.color());
    }

    #[test]
    fn percentage_color_medium() {
        let percentage = Percentage {
            used_percentage: Some(CONTEXT_THRESHOLD_HIGH as f64),
        };

        assert_eq!(Color::Fixed(BRIGHT_YELLOW), percentage.color());
    }

    #[test]
    fn percentage_output() {
        let percentage = Percentage {
            used_percentage: Some(10.0),
        };

        let output = format!("{percentage}");
        assert!(output.contains(CONTEXT_ICON));
        assert!(output.contains("10%"));
    }

    #[test]
    fn tokens_output() {
        let tokens = Tokens {
            total_input_tokens: Some(1234),
            total_output_tokens: Some(567),
        };

        let output = format!("{tokens}");
        assert!(output.contains(TOKENS_ICON));
        assert!(output.contains("1234↑"));
        assert!(output.contains("567↓"));
    }

    #[test]
    fn tokens_output_default() {
        let tokens = Tokens::default();

        let output = format!("{tokens}");
        assert!(output.contains(TOKENS_ICON));
        assert!(output.contains("0↑"));
        assert!(output.contains("0↓"));
    }
}
