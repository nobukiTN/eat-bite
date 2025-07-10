use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashSet;
use std::io;

// -------------------------
// ロジック関数
// -------------------------

fn input_guess(prompt: &str) -> Vec<u8> {
    loop {
        println!("{}", prompt);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("入力エラー");

        let trimmed = input.trim();
        if trimmed.len() != 3 || !trimmed.chars().all(|c| c.is_ascii_digit()) {
            println!("⚠️ エラー: 3桁の数字を入力してください。");
            continue;
        }

        let guess: Vec<u8> = trimmed.chars()
            .map(|c| c.to_digit(10).unwrap() as u8)
            .collect();

        let unique: std::collections::HashSet<_> = guess.iter().copied().collect();
        if unique.len() != 3 {
            println!("⚠️ エラー: 重複のない数字を使ってください。");
            continue;
        }

        return guess;
    }
}

fn input_secret(prompt: &str) -> Vec<u8> {
    println!("{}", prompt);
    input_guess("あなたの秘密の数字を入力してください >")
}


fn judge(answer: &Vec<u8>, guess: &Vec<u8>) -> (u8, u8) {
    let mut eat = 0;
    let mut bite = 0;

    for i in 0..3 {
        if answer[i] == guess[i] {
            eat += 1;
        } else if answer.contains(&guess[i]) {
            bite += 1;
        }
    }

    (eat, bite)
}

fn generate_answer() -> Vec<u8> {
    let mut digits: Vec<u8> = (0..=9).collect();
    let mut rng = thread_rng();
    digits.shuffle(&mut rng);
    digits[..3].to_vec()
}

fn generate_random_guess() -> Vec<u8> {
    generate_answer()
}

// -------------------------
// Bot構造体
// -------------------------

#[derive(Debug)]
struct GameBot {
    memory: Vec<(Vec<u8>, (u8, u8))>,
}

impl GameBot {
    fn new() -> Self {
        GameBot { memory: Vec::new() }
    }

    fn remember(&mut self, guess: &Vec<u8>, result: (u8, u8)) {
        self.memory.push((guess.clone(), result));
    }

    fn get_memory(&self) -> &Vec<(Vec<u8>, (u8, u8))> {
        &self.memory
    }

    fn filter_candidates(&self) -> Vec<Vec<u8>> {
        let mut all_candidates = vec![];

        for a in 0..10 {
            for b in 0..10 {
                if b == a { continue; }
                for c in 0..10 {
                    if c == a || c == b { continue; }
                    all_candidates.push(vec![a, b, c]);
                }
            }
        }

        let filtered: Vec<Vec<u8>> = all_candidates
            .into_iter()
            .filter(|candidate| {
                self.memory.iter().all(|(guess, result)| {
                    judge(candidate, guess) == *result
                })
            })
            .collect();

        filtered
    }

    fn generate_guess(&self) -> Vec<u8> {
        let candidates = self.filter_candidates();

        if candidates.is_empty() {
            panic!("候補が存在しません。記憶が矛盾している可能性があります。");
        }

        let mut rng = thread_rng();
        candidates.choose(&mut rng).unwrap().clone()
    }
}

// -------------------------
// テスト
// -------------------------

#[cfg(test)]
mod tests {
    use super::*;
#[test]
fn test_bot_filters_candidates_with_dbg() {
    let mut bot = GameBot::new();

    bot.remember(&vec![1, 2, 3], (0, 1)); // 1つだけBite
    bot.remember(&vec![4, 5, 6], (1, 0)); // 1つだけEat

    let candidates = bot.filter_candidates();

    dbg!(&candidates[..10.min(candidates.len())]); // 上位10個だけ表示
    dbg!(candidates.len());

    for candidate in candidates.iter() {
        for (past_guess, result) in bot.get_memory().iter() {
            assert_eq!(judge(candidate, past_guess), *result);
        }
    }
}


#[test]
fn test_bot_can_remember_guess_and_result() {
    let mut bot = GameBot::new();

    let guess1 = vec![1, 2, 3];
    let result1 = (1, 1);

    bot.remember(&guess1, result1);

    let memory = bot.get_memory();
    dbg!(&memory);

    assert_eq!(memory.len(), 1);
    assert_eq!(memory[0], (guess1.clone(), result1));

    let guess2 = vec![4, 5, 6];
    let result2 = (0, 2);

    bot.remember(&guess2, result2);
    dbg!(&bot);

    let memory = bot.get_memory();
    assert_eq!(memory.len(), 2);
    assert_eq!(memory[1], (guess2.clone(), result2));
}




    #[test]
    fn test_judge() {
        let answer = vec![5, 2, 7];

        assert_eq!(judge(&answer, &vec![5, 2, 7]), (3, 0));
        assert_eq!(judge(&answer, &vec![7, 5, 2]), (0, 3));
        assert_eq!(judge(&answer, &vec![5, 7, 2]), (1, 2));
        assert_eq!(judge(&answer, &vec![1, 3, 4]), (0, 0));
        assert_eq!(judge(&answer, &vec![7, 5, 1]), (0, 2));
    }

    #[test]
    fn test_generate_answer() {
        let answer = generate_answer();
        assert_eq!(answer.len(), 3);

        for &digit in &answer {
            assert!(digit <= 9);
        }

        let set: HashSet<u8> = answer.iter().copied().collect();
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_generate_random_guess() {
        let guess = generate_random_guess();
        assert_eq!(guess.len(), 3);

        for &digit in &guess {
            assert!(digit <= 9);
        }

        let unique: HashSet<u8> = guess.iter().copied().collect();
        assert_eq!(unique.len(), 3);
    }

   
}

// -------------------------
// 実行エントリポイント
// -------------------------



fn main() {
    let player_secret = input_secret("🔒 あなたの秘密の数字（3桁、例: 527）>");
    let bot_secret = generate_answer();
    let mut bot = GameBot::new();

    println!("🎮 対戦開始！ あなた vs Bot");

    let mut turn = 1;
    loop {
        println!("\n===== ターン {} =====", turn);

        // プレイヤーのターン
        let player_guess = input_guess("🧍 あなたの予想（3桁）>");
        let (eat_p, bite_p) = judge(&bot_secret, &player_guess);
        println!("🧍 あなたの予想: {:?} → {} Eat, {} Bite", player_guess, eat_p, bite_p);

        if eat_p == 3 {
            println!("🎉 あなたの勝利です！");
            break;
        }

        // Botのターン
        let bot_guess = bot.generate_guess();
        let (eat_b, bite_b) = judge(&player_secret, &bot_guess);
        println!("🤖 Botの予想: {:?} → {} Eat, {} Bite", bot_guess, eat_b, bite_b);

        if eat_b == 3 {
            println!("🤖 Botの勝利です！");
            break;
        }

        bot.remember(&bot_guess, (eat_b, bite_b));
        turn += 1;
    }
}
