use std::io::Write;
use std::time::Instant;
use wordle_solver::wordle::{INITIAL_BEST_WORD, Solver, Tile, get_result};

/// 解けなかった原因を表す
#[derive(Debug)]
enum FailReason {
    NoAnswer { turn: u32, guess: String },
    TooManyTurns,
}

/// 指定された答えに対してソルバーが何ターンで解けるかをシミュレートする。
fn simulate(answer: &str) -> Result<u32, FailReason> {
    let mut solver = Solver::new();
    let mut guess = INITIAL_BEST_WORD.to_string();

    for turn in 1u32.. {
        let result = get_result(&guess, answer);

        if result == [Tile::Green; 5] {
            return Ok(turn);
        }

        if turn >= 20 {
            return Err(FailReason::TooManyTurns);
        }

        match solver.give(&guess, &result) {
            Ok(Some(known)) => guess = known,
            Ok(None) => guess = solver.search().to_string(),
            Err(_) => {
                return Err(FailReason::NoAnswer {
                    turn,
                    guess: guess.clone(),
                });
            }
        }
    }
    unreachable!()
}

fn main() {
    let reference = Solver::new();
    let all_words = reference.all_words.clone();
    let total = all_words.len();

    println!("全{}単語をシミュレーション中...", total);
    println!("(--release ビルドを推奨: cargo run --bin benchmark --release)\n");

    let start = Instant::now();
    let mut total_turns: u64 = 0;
    let mut over6: Vec<(String, u32)> = Vec::new();
    let mut errors: Vec<(String, FailReason)> = Vec::new();
    let mut histogram = [0usize; 11]; // [0]=未使用, [1]~[9]=1~9ターン, [10]=10ターン以上

    for (i, answer) in all_words.iter().enumerate() {
        if (i + 1) % 50 == 0 || i + 1 == total {
            eprint!("\r進捗: {}/{}", i + 1, total);
            std::io::stderr().flush().ok();
        }

        match simulate(answer) {
            Ok(turns) => {
                total_turns += turns as u64;
                let idx = (turns as usize).min(10);
                histogram[idx] += 1;
                if turns > 6 {
                    over6.push((answer.to_string(), turns));
                }
            }
            Err(reason) => {
                errors.push((answer.to_string(), reason));
            }
        }
    }
    eprintln!();

    let elapsed = start.elapsed();
    let failed_count = over6.len() + errors.len();
    let solved_count = total - failed_count;
    let avg = total_turns as f64 / total as f64;

    println!("\n=== 結果 ===");
    println!("総単語数  : {}", total);
    println!("経過時間  : {:.1}秒", elapsed.as_secs_f64());
    println!("平均ターン: {:.4}", avg);
    println!(
        "6ターン以内: {} / {} ({:.2}%)",
        solved_count,
        total,
        solved_count as f64 / total as f64 * 100.0
    );

    println!("\nターン数分布:");
    for (turns, count) in histogram.iter().enumerate().skip(1) {
        if *count > 0 {
            let label = if turns >= 10 {
                "10+".to_string()
            } else {
                turns.to_string()
            };
            let bar: String = "#".repeat((*count).min(50));
            println!("  {}ターン: {:4}単語  {}", label, count, bar);
        }
    }

    if over6.is_empty() && errors.is_empty() {
        println!("\nすべての単語を6ターン以内に解けました！");
    } else {
        if !over6.is_empty() {
            println!("\n7ターン以上かかった単語: {}個", over6.len());
            for (word, turns) in &over6 {
                println!("  {} -> {}ターン", word, turns);
            }
        }
        if !errors.is_empty() {
            println!("\nエラーが発生した単語: {}個", errors.len());
            for (word, reason) in &errors {
                match reason {
                    FailReason::NoAnswer { turn, guess } => {
                        println!(
                            "  {} -> {}ターン目に\"{}\"を推測後、候補が0になった (ソルバーのバグ)",
                            word, turn, guess
                        );
                    }
                    FailReason::TooManyTurns => {
                        println!("  {} -> 20ターン超え", word);
                    }
                }
            }
        }
    }
}
