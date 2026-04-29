use std::cmp;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

const NUM_ALPHABET: usize = 26;

// aが0, zが25のusizeを返す
fn offset_from_a(c: char) -> usize {
    c as usize - 'a' as usize
}

#[derive(Clone)]
struct CharInfo {
    is_black: bool,
    num: usize,
    num_green: usize,
    yellow_indices: HashSet<usize>,
}

impl CharInfo {
    fn new() -> Self {
        Self {
            is_black: false,
            num: 0,
            num_green: 0,
            yellow_indices: HashSet::new(),
        }
    }
}

struct Knowledge {
    char_map: Vec<CharInfo>,
    num: usize,
    green_indices: HashSet<usize>,
}

impl Knowledge {
    fn new() -> Self {
        Self {
            char_map: vec![CharInfo::new(); NUM_ALPHABET],
            num: 0,
            green_indices: HashSet::new(),
        }
    }
}

pub enum StateError {
    InvalidInput,
    NoAnswer,
}

// ゲームの状態を持つ Solver 本体
pub struct State {
    pub possible_answers: HashSet<String>, // あり得るすべての答えを格納する
    char_and_pos_map: Vec<HashSet<String>>, // 文字と場所からそれを含む単語へのマップ
    char_and_num_map: Vec<HashSet<String>>, // 文字とその文字の出現数(ちょうどではなくそれ以上)から単語へのマップ
    knowledge: Knowledge,                   // 分かっている情報
}

impl State {
    // ファイルをロードして初期化した State を返す
    pub fn new() -> Self {
        let mut possible_answers = HashSet::new();
        let mut char_and_pos_map = vec![HashSet::new(); 5 * NUM_ALPHABET];
        let mut char_map = vec![HashSet::new(); NUM_ALPHABET];
        // 1単語に含まれる同じ文字の最大数は3
        let mut char_and_num_map = vec![HashSet::new(); 3 * NUM_ALPHABET];

        // ファイルを開く
        let file_path = "data/wordle-answers.txt";
        let file = File::open(file_path).expect("An error occurred opening file.");

        // ファイルを読み込む
        let reader = BufReader::new(file);

        // 1行(1単語)ずつ処理する
        for line in reader.lines() {
            let word = line.expect("An error occurred reading file.");

            // 全単語を入れる
            possible_answers.insert(word.clone());

            let mut count_map = vec![0; 26];
            for (i, c) in word.chars().enumerate() {
                // 1文字目の a なら index は 0
                // 1文字目の b なら index は 1
                // 2文字目の a なら index は 26
                char_and_pos_map[offset_from_a(c) + NUM_ALPHABET * i].insert(word.clone());

                char_map[offset_from_a(c)].insert(word.clone());

                char_and_num_map[offset_from_a(c) + count_map[offset_from_a(c)] * NUM_ALPHABET]
                    .insert(word.clone());
                count_map[offset_from_a(c)] += 1;
            }
        }

        Self {
            possible_answers,
            char_and_pos_map,
            char_and_num_map,
            knowledge: Knowledge::new(),
        }
    }

    // 1ターンの結果を与える
    // 答えが定まった場合には Some(ans) として返る
    pub fn give(&mut self, word: &str, res: &str) -> Result<Option<String>, StateError> {
        // 不正な入力の場合はエラーを返す
        if word.len() != 5 || res.len() != 5 {
            return Err(StateError::InvalidInput);
        }

        // word の各文字のカウントマップ
        // ただし、その文字が Black だった場合はカウントしない
        // 例えば、1と4文字目が同じで1文字目は Black , 4文字目は Green なことがあり得るが
        // この時に Black もカウントしていると num を 2に更新してしまう
        let mut count_map = [0; NUM_ALPHABET];

        for (i, (c, r)) in word.chars().zip(res.chars()).enumerate() {
            if !c.is_ascii_lowercase() {
                return Err(StateError::InvalidInput);
            }

            match r {
                // c が答えにない場合
                'b' => {
                    // c を含む単語を possible_answers から引く
                    self.possible_answers =
                        &self.possible_answers - &self.char_and_num_map[offset_from_a(c)];

                    let e = &mut self.knowledge.char_map[offset_from_a(c)];

                    e.is_black = true;
                    // c が単語に2文字以上あるが答えには1文字しかなかった場合、他のすべてが前にあるか、どれかが Green の場合はこれが Black になるが
                    // この文字のすべてが Green になっていない場合、これが Yellow であったと同等の情報として扱う必要がある
                    e.yellow_indices.insert(i);
                }
                // c はあるが場所が違う場合
                'y' => {
                    count_map[offset_from_a(c)] += 1;

                    // possible_answers と c を含む単語の積集合から、c の場所が一致する集合を引く
                    self.possible_answers = &self.possible_answers
                        & &(&self.char_and_num_map[offset_from_a(c)]
                            - &self.char_and_pos_map[offset_from_a(c) + i * NUM_ALPHABET]);

                    let e = &mut self.knowledge.char_map[offset_from_a(c)];
                    e.yellow_indices.insert(i);

                    if count_map[offset_from_a(c)] > e.num {
                        self.knowledge.num += 1;
                        e.num += 1;
                    }
                }
                // c の場所もあっている場合
                'g' => {
                    count_map[offset_from_a(c)] += 1;

                    // possible_answers と c の場所も一致する単語の積集合
                    self.possible_answers = &self.possible_answers
                        & &self.char_and_pos_map[offset_from_a(c) + i * NUM_ALPHABET];

                    let e = &mut self.knowledge.char_map[offset_from_a(c)];
                    // 未知の Green なら knowledge を更新
                    if self.knowledge.green_indices.insert(i) {
                        e.num_green += 1;
                        // Yellow にない Green なら num も更新
                        if e.num < e.num_green {
                            self.knowledge.num += 1;
                            e.num += 1;
                        }
                    }

                    if count_map[offset_from_a(c)] > e.num {
                        e.num += 1;
                    }
                }
                // 結果の入力がいずれでもなかった場合はエラーを返す
                _ => return Err(StateError::InvalidInput),
            }
        }

        let mut i = 0;
        while i < NUM_ALPHABET {
            let mut increment = true;

            let e = &mut self.knowledge.char_map[i];
            // 各 yellow_indices を更新(既にある Green の場所はあり得ないので Yellow と見なす)
            e.yellow_indices = &e.yellow_indices | &self.knowledge.green_indices;

            // 5と yellow_indices の要素数の差が探索中の Yellow の数と同じになった場合、Green が確定する
            if e.num > 0 && 5 - e.yellow_indices.len() == e.num - e.num_green {
                for j in 0..5 {
                    if !e.yellow_indices.contains(&j) {
                        self.possible_answers =
                            &self.possible_answers & &self.char_and_pos_map[i + j * NUM_ALPHABET];

                        self.knowledge.green_indices.insert(j);
                        e.num_green += 1;
                    }
                }
                // これが作動した場合は green_indices が更新されるので、ループをやり直す必要がある
                i = 0;
                increment = false;
            }

            // 各文字のあり得る最大数
            let possible_max = if e.is_black {
                e.num
            } else {
                cmp::min(
                    3,
                    5 - cmp::max(
                        self.knowledge.num - e.num,
                        e.yellow_indices.len() - e.num_green,
                    ),
                )
            };

            // 現在分かっている数と最大数から答えを絞る
            match (e.num, possible_max) {
                (1, 1) => {
                    self.possible_answers =
                        &self.possible_answers - &self.char_and_num_map[i + NUM_ALPHABET]
                }
                (1, 2) => {
                    self.possible_answers =
                        &self.possible_answers - &self.char_and_num_map[i + 2 * NUM_ALPHABET]
                }
                (2, 2) => {
                    self.possible_answers = &self.possible_answers
                        & &(&self.char_and_num_map[i + NUM_ALPHABET]
                            - &self.char_and_num_map[i + 2 * NUM_ALPHABET]);
                }
                (2, 3) => {
                    self.possible_answers =
                        &self.possible_answers & &self.char_and_num_map[i + NUM_ALPHABET];
                }
                (3, 3) => {
                    self.possible_answers =
                        &self.possible_answers & &self.char_and_num_map[i + 2 * NUM_ALPHABET];
                }
                (_, _) => (),
            }
            if increment {
                i += 1;
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
