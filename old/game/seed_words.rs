use std::{
    fs::File,
    io::{self, BufRead},
};

use rand::seq::SliceRandom;

pub struct SeedWords {
    inner: Vec<String>,
}

impl SeedWords {
    pub fn new() -> Self {
        let path = std::path::Path::new("assets/words.txt");
        let file = File::open(path).unwrap();
        let reader = io::BufReader::new(file);
        let words: Vec<String> = reader.lines().map_while(Result::ok).collect();

        Self { inner: words }
    }

    // TODO: Make this a [String; 25] ?
    pub fn get_random_words(&self, count: usize) -> Vec<String> {
        let random_words = self.inner.choose_multiple(&mut rand::thread_rng(), count);
        random_words.into_iter().cloned().collect()
    }
}
