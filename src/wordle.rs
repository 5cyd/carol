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
    is_black: bool,      // この文字で Black が出たか(true ならこれ以上 num は増えない)
    num: usize,          // 分かっている答えにあるこの文字の数
    num_green: usize,    // 分かっているこの文字の Green の数
    possible_max: usize, // 答えにあるこの文字の数のあり得る最大値
    // この文字で Yellow が出た位置
    // 「現時点で未発見と仮定した Yellow の Green にならない位置」という意味で使うので、この文字含むすべての分かっている Green がここにも含まれる
    yellow_indices: HashSet<usize>,
}

impl CharInfo {
    fn new() -> Self {
        Self {
            is_black: false,
            num: 0,
            num_green: 0,
            possible_max: 3,
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
pub struct Solver {
    pub possible_answers: HashSet<String>, // あり得るすべての答えを格納する
    char_and_pos_map: Vec<HashSet<String>>, // 文字と場所からそれを含む単語へのマップ
    char_and_num_map: Vec<HashSet<String>>, // 文字とその文字の出現数(ちょうどではなくそれ以上)から単語へのマップ
    knowledge: Knowledge,
    all_words: Vec<String>,
    answer_char_and_pos_map: Vec<usize>,
}

impl Solver {
    // ファイルをロードして初期化した Solver を返す
    pub fn new() -> Self {
        let mut possible_answers = HashSet::new();
        let mut char_and_pos_map = vec![HashSet::new(); 5 * NUM_ALPHABET];
        let mut char_map = vec![HashSet::new(); NUM_ALPHABET];
        // 1単語に含まれる同じ文字の最大数は3
        let mut char_and_num_map = vec![HashSet::new(); 3 * NUM_ALPHABET];
        let mut all_words = Vec::with_capacity(2315);

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

            all_words.push(word.clone());

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
            all_words,
            answer_char_and_pos_map: vec![0; 5 * NUM_ALPHABET],
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
        let mut green_and_yellow_count_map = vec![0; NUM_ALPHABET];

        // その文字が完全に Black と扱えるのは、その文字の数 = その文字の Black の数 となった時なので
        // それを判定するために black のカウントマップも必要となる
        let mut black_count_map = vec![0; NUM_ALPHABET];

        for (i, (c, r)) in word.chars().zip(res.chars()).enumerate() {
            if !c.is_ascii_lowercase() {
                return Err(StateError::InvalidInput);
            }

            match r {
                // c が答えにない場合
                'b' => {
                    black_count_map[offset_from_a(c)] += 1;

                    // c が単語に2文字以上あるが答えには1文字しかなかった場合、他のすべてが前にあるか、どれかが Green の場合はこれが Black になるが
                    // この場合、これが Yellow であったと同等の情報として扱う必要がある
                    if word.matches(c).count() > black_count_map[offset_from_a(c)] {
                        // 他に同じ文字があった場合は Yellow として処理する
                        self.possible_answers = &self.possible_answers
                            & &(&self.char_and_num_map[offset_from_a(c)]
                                - &self.char_and_pos_map[offset_from_a(c) + i * NUM_ALPHABET]);
                    } else {
                        // その文字を含む集合を引く
                        self.possible_answers =
                            &self.possible_answers - &self.char_and_num_map[offset_from_a(c)];
                    }
                    let e = &mut self.knowledge.char_map[offset_from_a(c)];
                    e.is_black = true;
                    e.yellow_indices.insert(i);
                }
                // c はあるが場所が違う場合
                'y' => {
                    green_and_yellow_count_map[offset_from_a(c)] += 1;

                    // possible_answers と c を含む単語の積集合から、c の場所が一致する集合を引く
                    self.possible_answers = &self.possible_answers
                        & &(&self.char_and_num_map[offset_from_a(c)]
                            - &self.char_and_pos_map[offset_from_a(c) + i * NUM_ALPHABET]);

                    let e = &mut self.knowledge.char_map[offset_from_a(c)];
                    e.yellow_indices.insert(i);

                    if green_and_yellow_count_map[offset_from_a(c)] > e.num {
                        self.knowledge.num += 1;
                        e.num += 1;
                    }
                }
                // c の場所もあっている場合
                'g' => {
                    green_and_yellow_count_map[offset_from_a(c)] += 1;

                    // possible_answers と c の場所も一致する単語の積集合
                    self.possible_answers = &self.possible_answers
                        & &self.char_and_pos_map[offset_from_a(c) + i * NUM_ALPHABET];

                    let e = &mut self.knowledge.char_map[offset_from_a(c)];
                    // 未知の Green なら knowledge を更新
                    if self.knowledge.green_indices.insert(i) {
                        e.num_green += 1;
                    }

                    if green_and_yellow_count_map[offset_from_a(c)] > e.num {
                        self.knowledge.num += 1;
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
            // 各 yellow_indices を更新(既にある Green の場所はあり得ないので Yellow と見なせる)
            e.yellow_indices = &e.yellow_indices | &self.knowledge.green_indices;

            // 5と yellow_indices の要素数の差が探索中の Yellow の数と同じになった場合、Green が確定する
            if e.num > e.num_green && 5 - e.yellow_indices.len() == e.num - e.num_green {
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

            // 各文字のあり得る最大数を調べる
            e.possible_max = if e.is_black {
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
            match (e.num, e.possible_max) {
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

        self.answer_char_and_pos_map = vec![0; 5 * NUM_ALPHABET];
        for s in &self.possible_answers {
            for (i, c) in s.chars().enumerate() {
                self.answer_char_and_pos_map[offset_from_a(c) + i * NUM_ALPHABET] += 1;
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

    // 最も絞れそうな単語を調べる
    pub fn search(&self) -> &str {
        // 評価関数
        let eval = |word: &str| {
            let mut ret = 0;
            for (i, c) in word.chars().enumerate() {
                let e = &self.knowledge.char_map[offset_from_a(c)];
                if e.is_black || e.num == e.possible_max {
                    continue;
                }

                let half = self.possible_answers.len() as i32 / 2;

                if !e.yellow_indices.contains(&i) {
                    let add = half as i32
                        - (half
                            - self.answer_char_and_pos_map[offset_from_a(c) + i * NUM_ALPHABET]
                                as i32)
                            .abs();
                    if e.num - e.num_green > 0 {
                        ret += add + half;
                    } else {
                        ret += add;
                    }
                }
            }
            ret
        };

        let mut best_eval_value = i32::MIN;
        let mut best_word = &self.all_words[0];

        for word in &self.all_words {
            let eval_value = eval(word);
            if eval_value > best_eval_value {
                best_eval_value = eval_value;
                best_word = word;
            }
        }
        best_word
    }
}
