#![allow(unused)]
pub fn kmp(st: &str, pat: &str) -> Vec<usize> {
    if st.is_empty() || pat.is_empty() || st.len() < pat.len() {
        return vec![];
    }

    let string = st.as_bytes();
    let pattern = pat.as_bytes();

    let mut partial = vec![0];
    for i in 1..pattern.len() {
        let mut j: usize = partial[i - 1];
        while j > 0 && pattern[j] != pattern[i] {
            j = partial[j - 1];
        }
        partial.push(if pattern[j] == pattern[i] { j + 1 } else { j });
    }
    println!("{:?}", partial);
    let mut ret = vec![];
    let mut j = 0;

    for (i, &c) in string.iter().enumerate() {
        while j > 0 && c != pattern[j] {
            j = partial[j - 1];
        }

        if c == pattern[j] {
            j += 1;
        }

        if j == pattern.len() {
            ret.push(i + 1 - j);
            j = partial[j - 1];
        }
    }
    ret
}

pub fn kmp2(st: &str, pat: &str) -> Vec<i8> {
    if st.is_empty() || pat.is_empty() || st.len() < pat.len() {
        return vec![];
    }

    let string = st.as_bytes();
    let pattern = pat.as_bytes();

    let mut partial: Vec<i8> = vec![-1; pattern.len() + 1];
    let mut k: i8 = -1;
    let mut j = 0;
    while j < pattern.len() {
        if k == -1 || pattern[j] == pattern[k as usize] {
            j += 1;
            k += 1;
            // partial.push(j);
            partial[j] = k;
        } else {
            k = partial[k as usize];
        }

    }
    println!("{:?}", partial);

    let mut ret = vec![];

    ret

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_letter_matches() {
        let index = kmp2("abacababdab", "abcabcmn");
        println!("{:?}", index);
        // debug_assert_eq!(index, vec![0, 1, 2]);
    }

    #[test]
    fn a_few_separate_matches() {
        let index = kmp("abababa", "ab");
        println!("{:?}", index);
        // assert_eq!(index, vec![0, 2, 4]);
    }

    #[test]
    fn one_match() {
        let index = kmp("ABC ABCDAB ABCDABCDABDE", "ABCDABD");
        assert_eq!(index, vec![15]);
    }

    #[test]
    fn lots_of_matches() {
        let index = kmp("aaabaabaaaaa", "aa");
        println!("{:?}", index);
        // assert_eq!(index, vec![0, 1, 4, 7, 8, 9, 10]);
    }

    #[test]
    fn lots_of_intricate_matches() {
        let index = kmp("ababababa", "aba");
        assert_eq!(index, vec![0, 2, 4, 6]);
    }

    #[test]
    fn not_found0() {
        let index = kmp("abcde", "f");
        assert_eq!(index, vec![]);
    }

    #[test]
    fn not_found1() {
        let index = kmp("abcde", "ac");
        assert_eq!(index, vec![]);
    }

    #[test]
    fn not_found2() {
        let index = kmp("ababab", "bababa");
        assert_eq!(index, vec![]);
    }

    #[test]
    fn empty_string() {
        let index = kmp("", "abcdef");
        assert_eq!(index, vec![]);
    }
}
