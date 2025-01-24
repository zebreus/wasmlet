pub(crate) fn rainbow_text(input: &str) -> Result<String, String> {
    // Define ANSI escape codes for rainbow colors
    let colors = [
        "\x1b[31m", // Red
        "\x1b[33m", // Yellow
        "\x1b[32m", // Green
        "\x1b[36m", // Cyan
        "\x1b[34m", // Blue
        "\x1b[35m", // Magenta
    ];

    if input.contains("\x1b") {
        return Err(
            "The input text already contains ANSI escape codes. I can't add color to that."
                .to_string(),
        );
    }

    let mut colorful_text = input
        .chars()
        .zip(colors.iter().cycle())
        .map(|(c, color)| {
            // Apply the color to the character
            format!("{}{}", color, c)
        })
        .collect::<String>();
    colorful_text.push_str("\x1b[0m"); // Reset the color at the end of the string

    Ok(colorful_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_text() {
        let result = rainbow_text("Hello, world!").unwrap();
        assert_eq!(
            result,
            "\x1b[31mH\x1b[33me\x1b[32ml\x1b[36ml\x1b[34mo\x1b[35m,\x1b[31m \x1b[33mw\x1b[32mo\x1b[36mr\x1b[34ml\x1b[35md\x1b[31m!\x1b[0m"
        );
    }

    #[test]
    fn fails_at_already_colored_text() {
        let result = rainbow_text("\x1b[31mred\x1b[0m");
        assert!(result.is_err());
    }
}
