use std::io;
use std::io::Write;

mod wordle;

fn main() {
    let mut solver = wordle::State::new();

    println!("Wordle solver by 5c");
    println!("Usage: Enter word and its result.");
    println!("Example: If you enter \"world\" and get ⬛🟨🟩🟨⬛, please type \"world bygyb\".");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Error reading input.");

        let input: Vec<&str> = input.split_whitespace().map(|s| s.trim()).collect();

        if input.len() != 2 {
            println!("Your input is invalid.");
            continue;
        }

        match solver.give(input[0], input[1]) {
            Err(wordle::StateError::InvalidInput) => println!("Your input is invalid."),
            Err(wordle::StateError::NoAnswer) => {
                println!("No way...");
                break;
            }
            Ok(opt) => match opt {
                Some(ans) => {
                    println!("The answer is \"{}\"!", ans);
                    break;
                }
                None => {
                    println!(
                        "The current number of possible answers is {}.",
                        solver.possible_answers.len()
                    );
                    if solver.possible_answers.len() >= 5 {
                        let v: Vec<_> = solver
                            .possible_answers
                            .iter()
                            .take(5)
                            .map(|s| s.as_str())
                            .collect();
                        println!("({}...)", v.join(", "));
                    } else {
                        let v: Vec<_> =
                            solver.possible_answers.iter().map(|s| s.as_str()).collect();
                        println!("({})", v.join(", "));
                    }
                }
            },
        }
    }
}
