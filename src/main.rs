use rand::seq::SliceRandom;
use rustyline::DefaultEditor;
use std::{collections::HashMap, fs, iter};

#[derive(Default)]
struct Brain {
    tokens: HashMap<Vec<String>, HashMap<String, usize>>,
}

impl Brain {
    const MAX_CONTEXT_SIZE: usize = 5;

    pub fn train(&mut self, text: &str) {
        let mut context: Vec<&str> = Vec::new();

        for token in Self::tokenize(text) {
            for cs in 1..=context.len() {
                let context = context[(context.len() - cs)..context.len()]
                    .iter()
                    .map(|token| token.to_string())
                    .collect();

                *self
                    .tokens
                    .entry(context)
                    .or_default()
                    .entry(token.to_string())
                    .or_default() += 1;
            }

            context.push(token);

            if context.len() > Self::MAX_CONTEXT_SIZE {
                context.remove(0);
            }
        }
    }

    pub fn prompt(&self, prompt: &str, length: usize) -> String {
        let mut out: Vec<_> = Self::tokenize(prompt).collect();
        let mut rng = rand::thread_rng();

        while out.len() < length {
            let mut next_token = None;

            for cs in (1..=Self::MAX_CONTEXT_SIZE).rev() {
                if cs > out.len() {
                    continue;
                }

                let context: Vec<_> = out[(out.len() - cs)..out.len()]
                    .iter()
                    .map(|token| token.to_string())
                    .collect();

                if let Some(next_tokens) = self.tokens.get(&context) {
                    let next_tokens: Vec<_> = next_tokens.iter().collect();

                    next_token = Some(
                        next_tokens
                            .choose_weighted(&mut rng, |(_token, frequency)| *frequency)
                            .unwrap()
                            .0,
                    );

                    break;
                }
            }

            if let Some(next_token) = next_token {
                out.push(next_token);
            } else {
                break;
            }
        }

        out.join("")
    }

    fn tokenize(s: &str) -> impl Iterator<Item = &str> {
        let mut chars = s.char_indices().peekable();

        iter::from_fn(move || loop {
            let (idx, ch) = chars.next()?;

            if ch.is_alphanumeric() {
                let idx_from = idx;
                let mut idx_to = idx + ch.len_utf8();

                while let Some((idx, ch)) = chars.peek() {
                    if ch.is_alphanumeric() {
                        idx_to = idx + ch.len_utf8();
                        chars.next();
                    } else {
                        break;
                    }
                }

                return Some(&s[idx_from..idx_to]);
            } else {
                let idx_from = idx;
                let idx_to = idx + ch.len_utf8();

                return Some(&s[idx_from..idx_to]);
            }
        })
    }
}

fn main() {
    println!("# chatgpt-at-home");
    println!();
    println!("Use `:train file.txt` to train the algorithm on given file; write");
    println!("anything else for the algorithm to respond; Ctrl-C to quit.");
    println!();
    println!("A few examples:");
    println!();
    println!("> :train sources/shakespeare.txt");
    println!("> ACT III");
    println!();
    println!("> :train sources/1984.txt");
    println!("> It was a");
    println!();
    println!("----");
    println!();

    let mut brain = Brain::default();
    let mut rl = DefaultEditor::new().unwrap();

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                if line.starts_with("> :train") {
                    println!("err: You don't have to write `> `, that's just the prompt sign used");
                    println!("     to distinguish between commands and algorithm's output.");
                    println!();
                    continue;
                }

                _ = rl.add_history_entry(&line);

                if let Some(file) = line.strip_prefix(":train ") {
                    match fs::read_to_string(file) {
                        Ok(text) => {
                            brain = Default::default();
                            brain.train(&text);
                        }

                        Err(err) => {
                            println!("err: {}", err);
                        }
                    }

                    println!();
                    continue;
                }

                println!("{}", brain.prompt(&line, 256));
                println!();
            }

            Err(_) => {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(
        "Hello, World!",
        &["Hello", ",", " ", "World", "!"]
    )]
    #[test_case(
        "#include <coffee.h>",
        &["#", "include", " ", "<", "coffee", ".", "h", ">"]
    )]
    #[test_case(
        "123 + 234 = 0xCAFEBABE",
        &["123", " ", "+", " ", "234", " ", "=", " ", "0xCAFEBABE"]
    )]
    fn tokenize(given: &str, expected: &[&str]) {
        let actual: Vec<_> = Brain::tokenize(given).collect();
        let expected: Vec<_> = expected.iter().map(|token| token.to_string()).collect();

        assert_eq!(expected, actual);
    }
}
