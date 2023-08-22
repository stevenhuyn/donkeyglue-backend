use std::{
    fs::File,
    io::{self, BufRead},
};

use rand::seq::SliceRandom;

pub struct WordBank {
    inner: Vec<String>,
}

impl WordBank {
    pub fn new() -> Self {
        let path = std::path::Path::new("assets/words.txt");
        let file = File::open(path).unwrap();
        let reader = io::BufReader::new(file);
        let words: Vec<String> = reader.lines().map_while(Result::ok).collect();

        Self { inner: words }
    }

    pub fn get_word_set(&self, count: usize) -> Vec<String> {
        let random_words = self.inner.choose_multiple(&mut rand::thread_rng(), count);
        random_words.into_iter().cloned().collect()
    }
}
