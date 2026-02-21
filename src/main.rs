use std::fmt;
use std::io;

use serde::Deserialize;

#[derive(Deserialize)]
struct ClaudeStatusLineData {}

impl fmt::Display for ClaudeStatusLineData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
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
    fn json_deserialization() {
        let json = r#"{}"#;

        let data: ClaudeStatusLineData = serde_json::from_str(json).unwrap();
        // TODO: This test should be changed to check an attribute once ClaudeStatusLineData has
        // one.
        assert_eq!("", format!("{data}"));
    }
}
