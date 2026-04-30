use std::io;
use std::io::Write;

mod wordle;

fn main() {
    let mut solver = wordle::Solver::new();

    println!("Wordle solver by 5c");
    println!("Enter word and its result.");
    println!("Example: If you enter \"world\" and get ⬛🟨🟩🟨⬛, please type \"world bygyb\".");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        // 入力を1行読み取る
        io::stdin()
            .read_line(&mut input)
            .expect("Error reading input.");

        // 入力を単語と結果に分けて空白をトリム
        let input: Vec<&str> = input.split_whitespace().map(|s| s.trim()).collect();

        // 空白がない or 3つ以上に分かれていた場合はやり直し
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
                // 答えが定まった場合は終了
                Some(ans) => {
                    println!("The answer is \"{}\"!", ans);
                    break;
                }
                None => {
                    // 答えの候補数を出力
                    println!(
                        "The current number of possible answers is {}.",
                        solver.possible_answers.len()
                    );
                    // 候補の単語を最大5個出力する
                    if solver.possible_answers.len() >= 5 {
                        let v: Vec<_> = solver
                            .possible_answers
                            .iter()
                            .take(5)
                            .map(|s| s.as_str())
                            .collect();
                        println!("({}, ...)", v.join(", "));
                    } else {
                        let v: Vec<_> =
                            solver.possible_answers.iter().map(|s| s.as_str()).collect();
                        println!("({})", v.join(", "));
                    }

                    // 次の最善手を探して出力する
                    let best_word = solver.search();
                    println!("Suggested next word: \"{}\"", best_word);
                }
            },
        }
    }
}
