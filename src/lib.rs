mod wordle;

use serde::Serialize;
use wasm_bindgen::prelude::*;
use wordle::StrExt;

#[derive(Serialize)]
struct GiveResult {
    answer: Option<String>,
    possible_count: Option<usize>,
    candidates: Option<Vec<String>>,
    next_word: Option<String>,
    error: Option<String>,
}

#[wasm_bindgen]
pub struct WasmSolver {
    inner: wordle::Solver,
}

#[wasm_bindgen]
impl WasmSolver {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: wordle::Solver::new(),
        }
    }

    pub fn initial_word() -> String {
        wordle::INITIAL_BEST_WORD.to_string()
    }

    pub fn give(&mut self, word: &str, result: &str) -> String {
        let tiles = match result.to_tiles() {
            Ok(t) => t,
            Err(_) => {
                return serde_json::to_string(&GiveResult {
                    answer: None,
                    possible_count: None,
                    candidates: None,
                    next_word: None,
                    error: Some(
                        "Invalid result. Use g (green), y (yellow), b (black).".to_string(),
                    ),
                })
                .unwrap();
            }
        };

        match self.inner.give(word, &tiles) {
            Err(wordle::StateError::InvalidInput) => serde_json::to_string(&GiveResult {
                answer: None,
                possible_count: None,
                candidates: None,
                next_word: None,
                error: Some("Invalid input.".to_string()),
            })
            .unwrap(),

            Err(wordle::StateError::NoAnswer) => serde_json::to_string(&GiveResult {
                answer: None,
                possible_count: None,
                candidates: None,
                next_word: None,
                error: Some("No way...".to_string()),
            })
            .unwrap(),

            Ok(Some(ans)) => serde_json::to_string(&GiveResult {
                answer: Some(ans),
                possible_count: Some(1),
                candidates: None,
                next_word: None,
                error: None,
            })
            .unwrap(),

            Ok(None) => {
                let count = self.inner.possible_answers.len();
                let candidates: Vec<String> = self
                    .inner
                    .possible_answers
                    .iter()
                    .take(5)
                    .cloned()
                    .collect();
                let next_word = self.inner.search().to_string();
                serde_json::to_string(&GiveResult {
                    answer: None,
                    possible_count: Some(count),
                    candidates: Some(candidates),
                    next_word: Some(next_word),
                    error: None,
                })
                .unwrap()
            }
        }
    }
}
