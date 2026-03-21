//! Command line program to generate a crossword puzzle from a list of words.
//!
//! ----------------------------------------------------------------------------

use std::io::stdin;

fn main() {
    let stdin = stdin();

    let words: Vec<String> = stdin.lines().flatten().collect();

    println!("Words: {words:?}");
}
