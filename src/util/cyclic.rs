const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

/// Generate a De Bruijn sequence of `length` bytes.
/// Every 4-byte subsequence is unique, making it trivial to locate
/// crash offsets.
pub fn cyclic(length: usize) -> Vec<u8> {
    let seq = de_bruijn(ALPHABET, 4);
    seq[..length.min(seq.len())].to_vec()
}

/// Find the offset of `subseq` in the cyclic pattern.
///
/// ```
/// use pwnkit::util::cyclic::{cyclic, cyclic_find};
///
/// let pattern = cyclic(200);
/// let needle = &pattern[44..48];
/// assert_eq!(cyclic_find(needle), Some(44));
/// ```
pub fn cyclic_find(subseq: &[u8]) -> Option<usize> {
    let seq = de_bruijn(ALPHABET, 4);
    seq.windows(subseq.len()).position(|w| w == subseq)
}

/// Martin's algorithm for generating a De Bruijn sequence
/// over `alphabet` with subsequences of length `n`.
fn de_bruijn(alphabet: &[u8], n: usize) -> Vec<u8> {
    let k = alphabet.len();
    let mut a = vec![0usize; k * n + 1];
    let mut seq: Vec<usize> = Vec::new();

    fn db(t: usize, p: usize, n: usize, k: usize, a: &mut [usize], seq: &mut Vec<usize>) {
        if t > n {
            if n % p == 0 {
                seq.extend_from_slice(&a[1..=p]);
            }
        } else {
            a[t] = a[t - p];
            db(t + 1, p, n, k, a, seq);
            for j in (a[t - p] + 1)..k {
                a[t] = j;
                db(t + 1, t, n, k, a, seq);
            }
        }
    }

    db(1, 1, n, k, &mut a, &mut seq);
    seq.into_iter().map(|i| alphabet[i]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cyclic_20() {
        assert_eq!(cyclic(20), b"aaaabaaacaaadaaaeaaa");
    }

    #[test]
    fn cyclic_32() {
        assert_eq!(cyclic(32), b"aaaabaaacaaadaaaeaaafaaagaaahaaa");
    }

    #[test]
    fn find_baaa() {
        assert_eq!(cyclic_find(b"baaa"), Some(4));
    }

    #[test]
    fn find_caaa() {
        assert_eq!(cyclic_find(b"caaa"), Some(8));
    }

    #[test]
    fn find_at_514() {
        let pat = cyclic(1000);
        assert_eq!(cyclic_find(&pat[514..518]), Some(514));
    }
}
