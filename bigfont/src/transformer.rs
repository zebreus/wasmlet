const FONT: &[u8] = include_bytes!("../Puffy.flf");
pub(crate) fn letter_text(input: &str) -> Result<String, String> {
    let font = figfont::FIGfont::read_from(FONT).map_err(|_| "Failed to read font".to_string())?;
    let output = input
        .chars()
        .map(|c| c as i32)
        .map(|c| {
            let character = font.get(c);
            let lines = character.lines();
            let string_lines = lines
                .iter()
                .map(|line| {
                    line.iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join("")
                })
                .collect::<Vec<_>>();
            string_lines
        })
        .fold(Vec::new(), |output, char_lines| {
            let mut output = output;
            for (i, line) in char_lines.iter().enumerate() {
                if i >= output.len() {
                    output.push(String::new());
                }
                output[i].push_str(&line);
            }
            output
        })
        .join("\n");

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_hello_world() {
        let result = letter_text("Hello, world!").unwrap();
        assert_eq!(
            result,
            " _   _         _    _                                         _        _  _ \n( ) ( )       (_ ) (_ )                                      (_ )     ( )( )\n| |_| |   __   | |  | |    _          _   _   _    _    _ __  | |    _| || |\n|  _  | /'__`\\ | |  | |  /'_`\\       ( ) ( ) ( ) /'_`\\ ( '__) | |  /'_` || |\n| | | |(  ___/ | |  | | ( (_) ) _    | \\_/ \\_/ |( (_) )| |    | | ( (_| || |\n(_) (_)`\\____)(___)(___)`\\___/'( )   `\\___x___/'`\\___/'(_)   (___)`\\__,_)(_)\n                               |/                                        (_)\n                                                                            "
        );
    }
}
