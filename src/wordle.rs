use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

const ALPHABET_NUM: usize = 26;

pub enum StateError {
    InvalidInput,
    NoAnswer,
}

// ゲームの状態を持つ Solver 本体
pub struct State {
    pub possible_answers: HashSet<String>, // あり得るすべての答えを格納する
    char_and_pos_map: Vec<HashSet<String>>, // 文字と場所からそれを含む単語へのマップ
    char_map: Vec<HashSet<String>>,        // 文字からそれを含む単語へのマップ
}

impl State {
    // ファイルをロードして初期化した GameState を返す
    pub fn new() -> Self {
        let mut possible_answers = HashSet::new();
        let mut char_and_pos_map = vec![HashSet::new(); ALPHABET_NUM * 5];
        let mut char_map = vec![HashSet::new(); ALPHABET_NUM];

        let file_path = "data/wordle-answers.txt";
        let file = File::open(file_path).expect("An error occurred opening file.");

        let reader = BufReader::new(file);

        for line in reader.lines() {
            let word = line.expect("An error occurred reading file.");

            // 全単語を入れる
            possible_answers.insert(word.clone());

            for (i, c) in word.chars().enumerate() {
                // 1文字目の a なら index は 0
                // 1文字目の b なら index は 1
                // 2文字目の a なら index は 26
                char_and_pos_map[c as usize - 'a' as usize + ALPHABET_NUM * i].insert(word.clone());
                char_map[c as usize - 'a' as usize].insert(word.clone());
            }
        }

        Self {
            possible_answers,
            char_and_pos_map,
            char_map,
        }
    }

    // 1ターンの結果を与える
    // 答えが定まった場合には Some(ans) として返る
    pub fn give(&mut self, word: &str, res: &str) -> Result<Option<String>, StateError> {
        // 不正な入力の場合はエラーを返す
        if word.len() != 5 || res.len() != 5 {
            return Err(StateError::InvalidInput);
        }

        for (i, (c, r)) in word.chars().zip(res.chars()).enumerate() {
            if (c as usize) < ('a' as usize) || (c as usize) > ('z' as usize) {
                return Err(StateError::InvalidInput);
            }
            match r {
                // c が答えにない場合
                'b' => {
                    // c を含む単語を possible_answers から引く
                    self.possible_answers =
                        &self.possible_answers - &self.char_map[c as usize - 'a' as usize];
                }
                // c はあるが場所が違う場合
                'y' => {
                    let offset = c as usize - 'a' as usize;
                    // possible_answers と c を含む単語の積集合から、c の場所が一致する集合を引く
                    self.possible_answers = &self.possible_answers
                        & &(&self.char_map[offset]
                            - &self.char_and_pos_map[offset + i * ALPHABET_NUM]);
                }
                // c の場所もあっている場合
                'g' => {
                    // possible_answers と c の場所も一致する単語の積集合
                    self.possible_answers = &self.possible_answers
                        & &self.char_and_pos_map[c as usize - 'a' as usize + i * ALPHABET_NUM];
                }
                // 結果の入力がいずれでもなかった場合はエラーを返す
                _ => return Err(StateError::InvalidInput),
            }
        }

        // possible_answers の要素が一つだけになったらそれを返す
        // 何かの間違いであり得る答えがなくなったらエラーを返す
        match self.possible_answers.len() {
            2.. => Ok(None),
            1.. => Ok(Some(
                self.possible_answers.iter().next().unwrap().to_owned(),
            )),
            0 => Err(StateError::NoAnswer),
        }
    }
}

#[test]
fn index_test() {
    assert_eq!('a' as usize - 'a' as usize, 0);
    assert_eq!('z' as usize - 'a' as usize, 25);
}
