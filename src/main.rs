mod config;

use std::fmt;
use std::io;

use ansi_term::Color;
use millisecond::prelude::*;
use serde::Deserialize;

use config::get_config;

// Colors
const BRIGHT_GREEN: u8 = 46;
const BRIGHT_YELLOW: u8 = 226;
const DODGER_BLUE: u8 = 39;
const LAVENDAR: u8 = 141;
const MAGENTA_PINK: u8 = 213;
const ORANGE: u8 = 208;
const PINK_RED: u8 = 203;

// Icons
const CONTEXT_ICON: &str = "🧠";
const COST_ICON: &str = "💰";
const DURATION_ICON: &str = "⏱️";
const MODEL_ICON: &str = "🤖";
const TOKENS_ICON: &str = "🪙";

const CONTEXT_BAR_WIDTH: usize = 10;
const CONTEXT_THRESHOLD_HIGH: i32 = 80; // Auto-compaction seems to kick in around 83%.
const CONTEXT_THRESHOLD_MEDIUM: i32 = 70;

#[derive(Deserialize)]
#[serde(from = "RawClaudeStatusLineData")]
struct ClaudeStatusLineData {
    cost: Amount,
    duration: Duration,
    model: Model,
    percentage: Percentage,
    tokens: Tokens,
}

impl fmt::Display for ClaudeStatusLineData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cost = self.cost;
        let duration = self.duration;
        let model = &self.model;
        let percentage = self.percentage;
        let tokens = self.tokens;

        write!(f, "{model} | {percentage} | {tokens} | {cost} | {duration}")
    }
}

impl From<RawClaudeStatusLineData> for ClaudeStatusLineData {
    fn from(raw: RawClaudeStatusLineData) -> Self {
        let context = raw.context_window.unwrap_or_default();
        let cost = raw.cost.unwrap_or_default();
        Self {
            cost: cost.amount,
            duration: cost.duration,
            model: raw.model,
            percentage: context.percentage,
            tokens: context.tokens,
        }
    }
}

#[derive(Deserialize)]
struct RawClaudeStatusLineData {
    cost: Option<Cost>,
    context_window: Option<ContextWindow>,
    model: Model,
}

#[derive(Deserialize, Default)]
struct ContextWindow {
    #[serde(flatten)]
    percentage: Percentage,
    #[serde(flatten)]
    tokens: Tokens,
}

#[derive(Deserialize, Default)]
struct Cost {
    #[serde(flatten)]
    amount: Amount,
    #[serde(flatten)]
    duration: Duration,
}

#[derive(Deserialize, Default, Clone, Copy)]
struct Amount {
    total_cost_usd: Option<f64>,
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let config = get_config().cost.clone().unwrap_or_default();
        let color = config.get_color_or(LAVENDAR);
        let icon = config.get_icon_or(COST_ICON);

        let cost = self.total_cost_usd.unwrap_or_default();
        let cost = Color::Fixed(color).paint(format!("${cost:.2}"));

        write!(f, "{icon} {cost}")
    }
}

#[derive(Deserialize, Default, Clone, Copy)]
struct Duration {
    total_api_duration_ms: Option<u64>,
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let config = get_config().duration.clone().unwrap_or_default();
        let color = config.get_color_or(DODGER_BLUE);
        let icon = config.get_icon_or(DURATION_ICON);

        let duration = Color::Fixed(color).paint(match self.total_api_duration_ms {
            Some(0) | None => "0s".to_string(),
            _ => {
                let ms = Millisecond::from(core::time::Duration::from_millis(
                    self.total_api_duration_ms.unwrap_or_default(),
                ));
                ms.pretty_with(MillisecondOption {
                    seconds: SecondsOptions::CombineWith {
                        precision: Some(0),
                        fixed_width: false,
                    },
                    ..Default::default()
                })
            }
        });

        write!(f, "{icon} {duration}")
    }
}

#[derive(Deserialize)]
struct Model {
    display_name: String,
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let config = get_config().model.clone().unwrap_or_default();
        let color = config.get_color_or(ORANGE);
        let icon = config.get_icon_or(MODEL_ICON);

        let display_name = Color::Fixed(color).paint(&self.display_name);

        write!(f, "{icon} {display_name}")
    }
}

#[derive(Deserialize, Default, Clone, Copy)]
struct Percentage {
    used_percentage: Option<f64>,
}

impl fmt::Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let config = get_config().percentage.clone().unwrap_or_default();
        let icon = config.get_icon_or(CONTEXT_ICON);

        let percent = self.used_percentage.unwrap_or_default() as i32;
        let filled = percent * CONTEXT_BAR_WIDTH as i32 / 100;
        let bar = "▓".repeat(filled as usize) + &"░".repeat(CONTEXT_BAR_WIDTH - filled as usize);
        let context = self.color().paint(format!("{bar} {percent}%"));

        write!(f, "{icon} {context}")
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
        let config = get_config().tokens.clone().unwrap_or_default();
        let color = config.get_color_or(MAGENTA_PINK);
        let icon = config.get_icon_or(TOKENS_ICON);

        let input = self.total_input_tokens.unwrap_or_default();
        let output = self.total_output_tokens.unwrap_or_default();
        let tokens = Color::Fixed(color).paint(format!("{input}↑ {output}↓"));

        write!(f, "{icon} {tokens}")
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
    fn cost_output() {
        let cost = Amount {
            total_cost_usd: Some(1.0),
        };

        let output = format!("{cost}");
        assert!(output.contains(COST_ICON));
        assert!(output.contains("$1.00"));
    }

    #[test]
    fn default_output() {
        let data = ClaudeStatusLineData {
            cost: Amount::default(),
            duration: Duration::default(),
            // Unlike the other fields, this one is required and has no default.
            model: Model {
                display_name: "Model Display Name".to_string(),
            },
            percentage: Percentage::default(),
            tokens: Tokens::default(),
        };

        let output = format!("{data}");
        assert!(output.contains(" 0%"));
        // TODO: We can't include a space here because the color code appears between the space and
        // the 0. It's unnecessary from a display perspective, but maybe the icons should be
        // included in the painted text.
        assert!(output.contains("0↑"));
        assert!(output.contains(" 0↓"));
        assert!(output.contains("$0.00"));
        // TODO: We can't include a space here because the color code appears between the space and
        // the 0. It's unnecessary from a display perspective, but maybe the icons should be
        // included in the painted text.
        assert!(output.contains("0s"));
    }

    #[test]
    fn duration_as_zero() {
        let duration = Duration {
            total_api_duration_ms: Some(0),
        };

        let output = format!("{duration}");
        assert!(output.contains("0s"));
    }

    #[test]
    fn duration_output() {
        let duration = Duration {
            total_api_duration_ms: Some(1_000),
        };

        let output = format!("{duration}");
        assert!(output.contains(DURATION_ICON));
        assert!(output.contains("1s"));
    }

    #[test]
    fn full_output() {
        let data = ClaudeStatusLineData {
            cost: Amount {
                total_cost_usd: Some(50.0),
            },
            duration: Duration {
                total_api_duration_ms: Some(60_000),
            },
            model: Model {
                display_name: "Model Display Name".to_string(),
            },
            percentage: Percentage {
                used_percentage: Some(20.0),
            },
            tokens: Tokens {
                total_input_tokens: Some(5),
                total_output_tokens: Some(10),
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
        assert!(output.contains(COST_ICON));
        assert!(output.contains("$50.00"));
        assert!(output.contains(DURATION_ICON));
        assert!(output.contains("1m"));
    }

    #[test]
    fn json_deserialization() {
        let json = r#"{
            "context_window": {"total_input_tokens": 10, "total_output_tokens": 5, "used_percentage": 1.0},
            "cost": {"total_api_duration_ms": 2500, "total_cost_usd": 50.0},
            "model": {"display_name": "Sonnet 4.5"}
        }"#;

        let data: ClaudeStatusLineData = serde_json::from_str(json).unwrap();
        assert_eq!("Sonnet 4.5", data.model.display_name);
        assert_eq!(Some(1.0), data.percentage.used_percentage);
        assert_eq!(Some(10), data.tokens.total_input_tokens);
        assert_eq!(Some(5), data.tokens.total_output_tokens);
        assert_eq!(Some(50.0), data.cost.total_cost_usd);
        assert_eq!(Some(2500), data.duration.total_api_duration_ms);
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
