use std::collections::HashSet;

pub struct NaiveSolver {
    words: Vec<String>,
}

impl Default for NaiveSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl NaiveSolver {
    pub fn new() -> Self {
        Self {
            words: include_str!("wordlist.txt")
                .split('\n')
                .map(|w| w.to_string())
                .collect(),
        }
    }
    /// This function answers the critical question of "What words could I make with a ouija board
    /// with only the given letters". Aka letters are allowed to repeat.
    pub fn valid_words_ouija<L>(&self, letters: L) -> Vec<String>
    where
        L: IntoIterator<Item = char>,
    {
        let mut found = vec![];
        let letters: HashSet<char> = letters.into_iter().collect::<HashSet<char>>();
        for word in &self.words {
            let word_chars: HashSet<char> = word.chars().collect();
            if word_chars.is_subset(&letters) {
                found.push(word.clone());
            }
        }
        found
    }
    /// This function answers the critical question of "What words could I make with a scabble board
    /// with only the given letters". Aka no repeated letter usage
    pub fn valid_words_scrabble<L>(&self, letters: L) -> Vec<String>
    where
        L: IntoIterator<Item = char>,
    {
        let mut found = vec![];
        let mut letters: Vec<char> = letters.into_iter().collect();
        letters.sort();
        'words: for word in &self.words {
            let mut these_letters = letters.clone();
            let mut word_chars: Vec<char> = word.chars().collect();
            word_chars.sort();
            for char in word_chars {
                if let Some(pos) = these_letters.iter().position(|c| c == &char) {
                    these_letters.swap_remove(pos);
                } else {
                    continue 'words;
                }
            }
            found.push(word.clone())
        }
        found
    }
}
