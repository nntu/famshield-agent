//! Vietnamese Input Method & Diacritics Normalizer (Telex & VNI to Canonical Latin).
//!
//! Converts raw keystrokes (Telex and VNI) and rendered accented Unicode text into a single
//! canonical unaccented Latin form for high-speed, O(1) multi-pattern matching via Aho-Corasick.

/// Checks if an ASCII character is a vowel.
#[inline]
pub fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

/// Checks if character is a Vietnamese diacritic mark or vowel with diacritics.
#[inline]
pub fn is_accented_char(c: char) -> bool {
    matches!(
        c,
        'đ' | 'Đ'
            | 'á'
            | 'à'
            | 'ả'
            | 'ã'
            | 'ạ'
            | 'ă'
            | 'ắ'
            | 'ằ'
            | 'ẳ'
            | 'ẵ'
            | 'ặ'
            | 'â'
            | 'ấ'
            | 'ầ'
            | 'ẩ'
            | 'ẫ'
            | 'ậ'
            | 'é'
            | 'è'
            | 'ẻ'
            | 'ẽ'
            | 'ẹ'
            | 'ê'
            | 'ế'
            | 'ề'
            | 'ể'
            | 'ễ'
            | 'ệ'
            | 'í'
            | 'ì'
            | 'ỉ'
            | 'ĩ'
            | 'ị'
            | 'ó'
            | 'ò'
            | 'ỏ'
            | 'õ'
            | 'ọ'
            | 'ô'
            | 'ố'
            | 'ồ'
            | 'ổ'
            | 'ỗ'
            | 'ộ'
            | 'ơ'
            | 'ớ'
            | 'ờ'
            | 'ở'
            | 'ỡ'
            | 'ợ'
            | 'ú'
            | 'ù'
            | 'ủ'
            | 'ũ'
            | 'ụ'
            | 'ư'
            | 'ứ'
            | 'ừ'
            | 'ử'
            | 'ữ'
            | 'ự'
            | 'ý'
            | 'ỳ'
            | 'ỷ'
            | 'ỹ'
            | 'ỵ'
    )
}

/// Maps accented Vietnamese Unicode characters to their base Latin equivalents.
pub fn fold_unicode_accents(c: char) -> char {
    match c {
        'đ' | 'Đ' => 'd',
        'á' | 'à' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' | 'â' | 'ấ' | 'ầ' | 'ẩ'
        | 'ẫ' | 'ậ' => 'a',
        'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => 'e',
        'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' => 'i',
        'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ớ' | 'ờ' | 'ở'
        | 'ỡ' | 'ợ' => 'o',
        'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => 'u',
        'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
        other => other,
    }
}

/// Decodes Vietnamese VNI input method into base unaccented Latin.
///
/// VNI rules:
/// - `d9` / `D9` -> `d`
/// - Circumflex / horn / breve: `a6` -> `a`, `e6` -> `e`, `o6` -> `o`, `o7` -> `o`, `u7` -> `u`, `a8` -> `a`
/// - Tones: `1` (sắc), `2` (huyền), `3` (hỏi), `4` (ngã), `5` (nặng)
///   When positioned after vowels (e.g. `sa1t`, `tu75`, `d9i5t`) or at word end (e.g. `chet1`, `danh1`),
///   these numbers are stripped.
pub fn decode_vni(input: &str) -> String {
    let mut words = Vec::new();

    for raw_word in input.split_whitespace() {
        let chars: Vec<char> = raw_word.chars().collect();
        let len = chars.len();
        if len == 0 {
            continue;
        }

        let mut out = String::with_capacity(len);
        let mut i = 0;
        let mut has_vowel = false;

        while i < len {
            let c = chars[i];
            let c_lower = c.to_ascii_lowercase();

            if is_vowel(c_lower) || is_accented_char(c) {
                has_vowel = true;
            }

            // Check d9 -> d
            if c_lower == 'd' && i + 1 < len && chars[i + 1] == '9' {
                out.push('d');
                i += 2;
                continue;
            }

            // Check vowel + 6 (â, ê, ô)
            if (c_lower == 'a' || c_lower == 'e' || c_lower == 'o')
                && i + 1 < len
                && chars[i + 1] == '6'
            {
                out.push(c_lower);
                has_vowel = true;
                i += 2;
                continue;
            }

            // Check vowel + 7 (ơ, ư, or variant a7)
            if (c_lower == 'o' || c_lower == 'u' || c_lower == 'a')
                && i + 1 < len
                && chars[i + 1] == '7'
            {
                out.push(c_lower);
                has_vowel = true;
                i += 2;
                continue;
            }

            // Check a8 (ă)
            if c_lower == 'a' && i + 1 < len && chars[i + 1] == '8' {
                out.push('a');
                has_vowel = true;
                i += 2;
                continue;
            }

            // Check tone numbers (1..=5)
            if matches!(c, '1' | '2' | '3' | '4' | '5') {
                // If word has a vowel and this digit is after a letter or after another VNI modifier (6, 7, 8, 9)
                if has_vowel
                    && (i > 0
                        && (chars[i - 1].is_alphabetic()
                            || matches!(chars[i - 1], '6' | '7' | '8' | '9')))
                {
                    // Skip the tone number
                    i += 1;
                    continue;
                }
            }

            out.push(c);
            i += 1;
        }

        words.push(out);
    }

    words.join(" ")
}

/// Decodes Vietnamese Telex input method into base unaccented Latin.
///
/// Telex rules:
/// - Double letters: `dd` -> `d`, `aa` -> `a`, `ee` -> `e`, `oo` -> `o`
/// - Horns / Breves: `aw` -> `a`, `ow` -> `o`, `uw` -> `u`, `w` -> `u` (when after vowel)
/// - Tone marks: `s` (sắc), `f` (huyền), `r` (hỏi), `x` (ngã), `j` (nặng), `z` (xóa dấu)
///   When positioned after a vowel or at the end of a syllable containing a vowel,
///   these characters are tone marks and are stripped.
pub fn decode_telex(input: &str) -> String {
    let mut words = Vec::new();

    for raw_word in input.split_whitespace() {
        let chars: Vec<char> = raw_word.chars().collect();
        let len = chars.len();
        if len == 0 {
            continue;
        }

        let mut out = String::with_capacity(len);
        let mut i = 0;
        let mut has_vowel = false;

        // Special word preservation: "sex", "six", "sox"
        let lower_word: String = chars.iter().map(|c| c.to_ascii_lowercase()).collect();
        if lower_word == "sex" || lower_word == "six" || lower_word == "sox" {
            words.push(lower_word);
            continue;
        }

        while i < len {
            let c = chars[i];
            let c_lower = c.to_ascii_lowercase();

            if is_vowel(c_lower) || is_accented_char(c) {
                has_vowel = true;
            }

            // dd -> d
            if c_lower == 'd' && i + 1 < len && chars[i + 1].eq_ignore_ascii_case(&'d') {
                out.push('d');
                i += 2;
                continue;
            }

            // aa -> a, aw -> a
            if c_lower == 'a' && i + 1 < len {
                let next = chars[i + 1].to_ascii_lowercase();
                if next == 'a' || next == 'w' {
                    out.push('a');
                    has_vowel = true;
                    i += 2;
                    continue;
                }
            }

            // ee -> e
            if c_lower == 'e' && i + 1 < len && chars[i + 1].eq_ignore_ascii_case(&'e') {
                out.push('e');
                has_vowel = true;
                i += 2;
                continue;
            }

            // oo -> o, ow -> o
            if c_lower == 'o' && i + 1 < len {
                let next = chars[i + 1].to_ascii_lowercase();
                if next == 'o' || next == 'w' {
                    out.push('o');
                    has_vowel = true;
                    i += 2;
                    continue;
                }
            }

            // uw -> u
            if c_lower == 'u' && i + 1 < len && chars[i + 1].eq_ignore_ascii_case(&'w') {
                out.push('u');
                has_vowel = true;
                i += 2;
                continue;
            }

            // standalone w after a consonant or vowel (e.g. "tuw" -> "tu", "w" -> "u")
            if c_lower == 'w' {
                if has_vowel {
                    // skip w modifier (e.g. "tuw" -> "tu")
                    i += 1;
                    continue;
                } else {
                    out.push('u');
                    has_vowel = true;
                    i += 1;
                    continue;
                }
            }

            // Tone keys: s, f, r, x, j, z
            // Condition: Must have encountered a vowel earlier in this syllable
            if matches!(c_lower, 's' | 'f' | 'r' | 'x' | 'j' | 'z') && has_vowel {
                // If it is the last character of the word (e.g. "cheets", "ddanhs", "ddijt", "mex")
                if i + 1 == len {
                    i += 1;
                    continue;
                }

                // Or if it is immediately after a vowel and followed by a coda consonant (e.g. "cheest", "ddijch", "dijt")
                if i > 0 && is_vowel(chars[i - 1].to_ascii_lowercase()) {
                    let next = chars[i + 1].to_ascii_lowercase();
                    if next.is_alphabetic() && !is_vowel(next) {
                        i += 1;
                        continue;
                    }
                }
            }

            out.push(c_lower);
            i += 1;
        }

        words.push(out);
    }

    words.join(" ")
}

/// Normalizes text from all typing modes (Telex, VNI, Unicode, Leetspeak) into a Canonical Unaccented Latin Form.
///
/// Pipeline:
/// 1. Remove obfuscation punctuation (dots, hyphens, underscores between letters).
/// 2. Decode VNI numbers (`d9`, `e6`, `u7`, `a8`, trailing tone numbers `1..=5`).
/// 3. Decode Telex combinations (`dd`, `ee`, `oo`, `aa`, `aw`, `ow`, `uw`, tone keys `s,f,r,x,j`).
/// 4. Fold all accented Unicode diacritics into base Latin.
/// 5. Convert common leetspeak digits (`0` -> `o`, `3` -> `e`, `4` -> `a`, `7` -> `t`).
/// 6. Collapse redundant spaces and trim.
pub fn canonical_vietnamese(raw: &str) -> String {
    if raw.is_empty() {
        return String::new();
    }

    // Step 1: Strip obfuscating punctuation separating characters (e.g. "c.h.e.t" -> "chet", "d_m" -> "dm")
    let mut deobfuscated = String::with_capacity(raw.len());
    let chars: Vec<char> = raw.chars().collect();
    for i in 0..chars.len() {
        let ch = chars[i];
        if matches!(
            ch,
            '.' | '_' | '-' | '*' | '@' | '#' | '!' | '~' | '/' | '\\' | '|'
        ) {
            // If surrounded by alphanumeric letters, strip delimiter to defeat obfuscation
            let has_prev = i > 0 && chars[i - 1].is_alphanumeric();
            let has_next = i + 1 < chars.len() && chars[i + 1].is_alphanumeric();
            if has_prev && has_next {
                continue;
            }
        }
        deobfuscated.push(ch);
    }

    // Step 2: Collapse isolated single letters separated by space (e.g. "c h e e t s" -> "cheets", "v c l" -> "vcl")
    let collapsed_singles = collapse_spaced_single_letters(&deobfuscated);

    // Step 3: Decode VNI input
    let after_vni = decode_vni(&collapsed_singles);

    // Step 4: Decode Telex input
    let after_telex = decode_telex(&after_vni);

    // Step 5: Fold Unicode accents and map remaining leet speak
    let mut canonical = String::with_capacity(after_telex.len());
    for ch in after_telex.chars() {
        let ch_lower = ch.to_ascii_lowercase();
        let folded = fold_unicode_accents(ch_lower);

        match folded {
            // Leetspeak replacements
            '0' => canonical.push('o'),
            '3' => canonical.push('e'),
            '4' => canonical.push('a'),
            '7' => canonical.push('t'),
            c if c.is_whitespace() => {
                if !canonical.ends_with(' ') {
                    canonical.push(' ');
                }
            }
            c if c.is_alphanumeric() => {
                canonical.push(c);
            }
            _ => {}
        }
    }

    canonical.trim().to_string()
}

/// Collapses sequences of isolated single-letter tokens separated by spaces into compact acronyms.
/// E.g. "thang d m may" -> "thang dm may", "v c l" -> "vcl", but leaves "admin" as "admin".
pub fn collapse_spaced_single_letters(text: &str) -> String {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.is_empty() {
        return String::new();
    }

    let mut result = Vec::new();
    let mut current_single_run = String::new();

    for token in tokens {
        if token.chars().count() == 1 && token.chars().next().is_some_and(|c| c.is_alphabetic()) {
            current_single_run.push_str(token);
        } else {
            if !current_single_run.is_empty() {
                result.push(current_single_run.clone());
                current_single_run.clear();
            }
            result.push(token.to_string());
        }
    }

    if !current_single_run.is_empty() {
        result.push(current_single_run);
    }

    result.join(" ")
}

/// Verifies whether a matched substring represents a distinct word boundary.
/// Crucial for preventing false positives on short tokens (e.g. "dm" in "admin", "cc" in "account").
pub fn is_word_boundary(text: &str, start_idx: usize, end_idx: usize) -> bool {
    let before = text[..start_idx].chars().next_back();
    let after = text[end_idx..].chars().next();

    let before_ok = match before {
        None => true,
        Some(c) => !c.is_alphanumeric(),
    };

    let after_ok = match after {
        None => true,
        Some(c) => !c.is_alphanumeric(),
    };

    before_ok && after_ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vni_decoding() {
        assert_eq!(decode_vni("d9anh1"), "danh");
        assert_eq!(decode_vni("che6t1"), "chet");
        assert_eq!(decode_vni("tu75 sa1t"), "tu sat");
        assert_eq!(decode_vni("d9i5t"), "dit");
        assert_eq!(decode_vni("kho3a tha7n"), "khoa than");
    }

    #[test]
    fn test_telex_decoding() {
        assert_eq!(decode_telex("ddanhs"), "danh");
        assert_eq!(decode_telex("cheets"), "chet");
        assert_eq!(decode_telex("tuws satj"), "tu sat");
        assert_eq!(decode_telex("ddijt"), "dit");
        assert_eq!(decode_telex("dijch"), "dich");
        assert_eq!(decode_telex("gaiw goij"), "gai goi");
        // Special words like sex must be preserved
        assert_eq!(decode_telex("sex"), "sex");
    }

    #[test]
    fn test_canonical_vietnamese_multi_engine() {
        // 1. Telex
        assert_eq!(canonical_vietnamese("c h e e t s"), "chet");
        assert_eq!(canonical_vietnamese("cheets"), "chet");
        assert_eq!(canonical_vietnamese("ddanhs"), "danh");
        assert_eq!(canonical_vietnamese("tuws satj"), "tu sat");

        // 2. VNI
        assert_eq!(canonical_vietnamese("che6t1"), "chet");
        assert_eq!(canonical_vietnamese("d9anh1"), "danh");
        assert_eq!(canonical_vietnamese("tu75 sa1t"), "tu sat");
        assert_eq!(canonical_vietnamese("d9i5t"), "dit");

        // 3. Rendered Unicode
        assert_eq!(canonical_vietnamese("chết"), "chet");
        assert_eq!(canonical_vietnamese("đánh"), "danh");
        assert_eq!(canonical_vietnamese("tự sát"), "tu sat");
        assert_eq!(canonical_vietnamese("địt"), "dit");

        // 4. Obfuscation & Leetspeak
        assert_eq!(canonical_vietnamese("c.h.3.t"), "chet");
        assert_eq!(canonical_vietnamese("d_m"), "dm");
        assert_eq!(canonical_vietnamese("s-e-x"), "sex");
    }

    #[test]
    fn test_word_boundary() {
        assert!(is_word_boundary("dm", 0, 2));
        assert!(is_word_boundary("thang dm may", 6, 8));
        assert!(!is_word_boundary("admin", 1, 3)); // "dm" inside "admin"
        assert!(!is_word_boundary("account", 1, 3)); // "cc" inside "account"
    }
}
