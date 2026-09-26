use std::path::PathBuf;

use mundareu_game::{AppExit, RunOptions};

fn main() -> AppExit {
    mundareu_game::run_with(parse_arguments(std::env::args().skip(1)))
}

fn parse_arguments(arguments: impl IntoIterator<Item = String>) -> RunOptions {
    let mut options = RunOptions::default();
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--screenshot" => options.screenshot_path = arguments.next().map(PathBuf::from),
            "--exit-after-seconds" => {
                options.exit_after_seconds = arguments.next().and_then(|value| value.parse().ok());
            }
            "--log-frame-rate" => options.log_frame_rate = true,
            unknown => eprintln!("ignoring unknown argument {unknown}"),
        }
    }
    options
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(arguments: &[&str]) -> RunOptions {
        parse_arguments(arguments.iter().map(|argument| argument.to_string()))
    }

    #[test]
    fn no_arguments_run_the_plain_game() {
        assert_eq!(parse(&[]), RunOptions::default());
    }

    #[test]
    fn every_flag_fills_its_option() {
        let options = parse(&[
            "--screenshot",
            "proof.png",
            "--exit-after-seconds",
            "4.5",
            "--log-frame-rate",
        ]);
        assert_eq!(
            options,
            RunOptions {
                screenshot_path: Some(PathBuf::from("proof.png")),
                exit_after_seconds: Some(4.5),
                log_frame_rate: true,
            }
        );
    }

    #[test]
    fn unknown_arguments_and_unparsable_numbers_are_ignored() {
        let options = parse(&["--mystery", "--exit-after-seconds", "soon"]);
        assert_eq!(options, RunOptions::default());
    }
}
