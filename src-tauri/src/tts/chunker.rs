//! Splitting text into speakable chunks.
//!
//! Sending an entire article to a TTS provider in one request means waiting 5-10
//! seconds before any sound comes out. Splitting on sentence boundaries and keeping the
//! very first chunk short instead means the caller can start synthesizing chunk N+1
//! while chunk N is already playing — first sound in roughly a second, the rest
//! streaming in behind it.

use std::collections::VecDeque;

/// Splits `text` into chunks no longer than `first_max` (chunk 0) / `rest_max` (every
/// chunk after) characters, breaking on sentence boundaries where possible.
///
/// A sentence longer than the limit on its own is hard-wrapped at the nearest
/// whitespace rather than mid-word. Chunks that would be empty (blank lines, runs of
/// whitespace) are dropped.
pub fn split(text: &str, first_max: usize, rest_max: usize) -> Vec<String> {
    let first_max = first_max.max(1);
    let rest_max = rest_max.max(1);

    let mut atoms: VecDeque<String> = split_sentences(text).into_iter().collect();
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();

    while let Some(atom) = atoms.pop_front() {
        let limit = if chunks.is_empty() { first_max } else { rest_max };

        if current.is_empty() {
            if atom.chars().count() <= limit {
                current = atom;
            } else {
                let (head, tail) = wrap_at(&atom, limit);
                current = head;
                if !tail.is_empty() {
                    atoms.push_front(tail);
                }
            }
            continue;
        }

        let joined_len = current.chars().count() + 1 + atom.chars().count();
        if joined_len <= limit {
            current.push(' ');
            current.push_str(&atom);
        } else {
            chunks.push(std::mem::take(&mut current));
            atoms.push_front(atom);
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// Breaks `text` into sentence-sized pieces on `.` `!` `?` `…` and line breaks, trimmed
/// and with blank pieces dropped.
///
/// The tricky part is telling a real sentence end from an abbreviation
/// (`v.v.`, `TP.HCM`, `Mr. Nam`) — this is a heuristic, not a parser, and gets those
/// wrong occasionally on purpose rather than trying to special-case every abbreviation:
/// a period is *not* treated as an ending when the character right before it is an
/// uppercase letter (`TP.`, `A.`), or when the character right after it (skipping
/// spaces) is lowercase (`v.v.`, `e.g. rest`).
fn split_sentences(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut sentences = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];

        if c == '\n' {
            push_trimmed(&mut sentences, &chars[start..i]);
            start = i + 1;
            i += 1;
            continue;
        }

        if matches!(c, '.' | '!' | '?' | '…') {
            let is_period = c == '.';
            let should_break = if is_period {
                let prev_is_upper = i > 0 && chars[i - 1].is_ascii_uppercase();
                let mut j = i + 1;
                while j < chars.len() && chars[j] == ' ' {
                    j += 1;
                }
                let next_is_lower = j < chars.len() && chars[j].is_lowercase();
                !prev_is_upper && !next_is_lower
            } else {
                true
            };

            if should_break {
                let mut end = i + 1;
                // Swallow a run of terminators together (`?!`, `...`) as one boundary.
                while end < chars.len() && matches!(chars[end], '.' | '!' | '?' | '…') {
                    end += 1;
                }
                push_trimmed(&mut sentences, &chars[start..end]);
                start = end;
                i = end;
                continue;
            }
        }

        i += 1;
    }

    push_trimmed(&mut sentences, &chars[start..]);
    sentences
}

fn push_trimmed(sentences: &mut Vec<String>, chars: &[char]) {
    let piece: String = chars.iter().collect::<String>().trim().to_string();
    if !piece.is_empty() {
        sentences.push(piece);
    }
}

/// Cuts `s` at the last whitespace at or before `limit` characters, so the head never
/// ends mid-word. Falls back to a hard cut at `limit` only when `s` has no whitespace
/// in range at all (one very long token) — the only case where "don't cut mid-word"
/// cannot be honoured and something still has to give.
fn wrap_at(s: &str, limit: usize) -> (String, String) {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= limit {
        return (s.to_string(), String::new());
    }

    let search_end = limit.min(chars.len());
    let space_at = (0..search_end).rev().find(|&i| chars[i].is_whitespace());
    let cut = match space_at {
        Some(i) if i > 0 => i,
        _ => search_end.max(1),
    };

    let head: String = chars[..cut].iter().collect::<String>().trim().to_string();
    let tail: String = chars[cut..].iter().collect::<String>().trim().to_string();
    (head, tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_produces_no_chunks() {
        assert!(split("", 120, 280).is_empty());
        assert!(split("   \n  ", 120, 280).is_empty());
    }

    #[test]
    fn a_short_vietnamese_sentence_is_one_chunk() {
        let chunks = split("Xin chào, hôm nay trời đẹp.", 120, 280);
        assert_eq!(chunks, vec!["Xin chào, hôm nay trời đẹp."]);
    }

    #[test]
    fn text_with_no_trailing_punctuation_is_still_captured() {
        let chunks = split("không có dấu câu cuối", 120, 280);
        assert_eq!(chunks, vec!["không có dấu câu cuối"]);
    }

    #[test]
    fn abbreviation_v_v_is_not_split_in_the_middle() {
        let chunks = split(
            "Có nhiều thứ như sách, bút, v.v. Chúng ta nên mua trước khi đi.",
            280,
            280,
        );
        assert_eq!(chunks.len(), 1, "{chunks:?}");
        assert!(chunks[0].contains("v.v."), "{chunks:?}");
    }

    #[test]
    fn abbreviation_before_a_place_name_is_not_split() {
        let chunks = split("Tôi sống ở TP.HCM từ nhỏ.", 280, 280);
        assert_eq!(chunks, vec!["Tôi sống ở TP.HCM từ nhỏ."]);
    }

    #[test]
    fn multiple_sentences_merge_up_to_the_limit() {
        let chunks = split("Câu một. Câu hai. Câu ba.", 280, 280);
        assert_eq!(chunks, vec!["Câu một. Câu hai. Câu ba."]);
    }

    #[test]
    fn the_first_chunk_respects_its_own_shorter_limit() {
        let text = "Câu đầu tiên khá là dài để vượt ngưỡng mười ký tự cho phép. \
                     Câu thứ hai cũng dài không kém, để kiểm tra ngưỡng chung ba trăm.";
        let chunks = split(text, 10, 300);
        assert!(chunks[0].chars().count() <= 10 || !chunks[0].contains(' '), "{chunks:?}");
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn a_sentence_longer_than_the_limit_is_wrapped_at_whitespace_not_mid_word() {
        let text = "một hai ba bốn năm sáu bảy tám chín mười mười một mười hai mười ba mười bốn.";
        let chunks = split(text, 20, 20);
        assert!(chunks.len() >= 2, "{chunks:?}");
        for chunk in &chunks {
            assert!(chunk.chars().count() <= 20, "chunk too long: {chunk:?}");
        }
        // Rejoining the chunks recovers every word (no character was dropped or
        // duplicated at a wrap boundary, and no word was cut in half).
        let rejoined = chunks.join(" ");
        assert_eq!(rejoined.split_whitespace().count(), text.split_whitespace().count());
    }

    /// The one case where "don't cut mid-word" cannot be honoured: a single token with
    /// no whitespace in it at all, longer than the limit on its own. Falls back to a
    /// hard cut rather than looping forever or producing a chunk over the limit.
    #[test]
    fn a_single_token_with_no_whitespace_falls_back_to_a_hard_cut_without_panicking() {
        let text = "mười_một_không_có_khoảng_trắng_ở_đây_luôn_dài_hơn_hai_mươi_ký_tự.";
        let chunks = split(text, 20, 20);
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(chunk.chars().count() <= 20, "chunk too long: {chunk:?}");
        }
        // No character lost across the hard cuts.
        assert_eq!(chunks.concat().chars().count(), text.chars().count());
    }

    #[test]
    fn a_two_thousand_character_article_starts_with_a_short_first_chunk() {
        let sentence = "Đây là một câu tiếng Việt có dấu để kiểm tra việc cắt đoạn dài. ";
        let text = sentence.repeat(2000 / sentence.len() + 1);
        let chunks = split(&text, 120, 280);
        assert!(!chunks.is_empty());
        assert!(
            chunks[0].chars().count() <= 120,
            "first chunk was {} chars",
            chunks[0].chars().count()
        );
        for chunk in &chunks[1..] {
            assert!(chunk.chars().count() <= 280, "chunk too long: {chunk:?}");
        }
    }

    #[test]
    fn splitting_never_drops_or_duplicates_words() {
        let text = "Gặp anh A. Nguyễn ở đó.";
        let chunks = split(text, 280, 280);
        let total_words: usize = chunks.iter().map(|c| c.split_whitespace().count()).sum();
        assert_eq!(total_words, text.split_whitespace().count());
    }
}
