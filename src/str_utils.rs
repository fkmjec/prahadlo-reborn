use std::cmp::min;

// works pretty fast for short strings only because of UTF-8
pub fn get_common_prefix_len(s1: &String, s2: &String) -> usize {
    let maximum = min(s1.len(), s2.len());
    let mut index = 0;
    while index < maximum {
        if s1.chars().nth(index) != s2.chars().nth(index) {
            break;
        }
        index += 1;
    }
    index
}