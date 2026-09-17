use crate::models::Track;
use std::collections::HashSet;

/// Normalizes a string by converting to lowercase, removing common noise tags
/// like '(Official Video)', '[Audio]', '(Remastered)', punctuation, and extra whitespace.
pub fn normalize_string(input: &str) -> String {
    let lower = input.to_lowercase();
    let mut cleaned = String::with_capacity(lower.len());

    // Filter out common suffixes / noise keywords
    let noise_patterns = [
        "official video",
        "official audio",
        "official music video",
        "lyric video",
        "lyrics",
        "audio",
        "remastered",
        "remaster",
        "hd",
        "hq",
        "4k",
        "feat.",
        "feat",
        "ft.",
        "ft",
    ];

    let mut working = lower;
    for pattern in noise_patterns {
        working = working.replace(pattern, "");
    }

    // Keep alphanumeric and single spaces
    for ch in working.chars() {
        if ch.is_alphanumeric() || ch == ' ' {
            cleaned.push(ch);
        } else {
            cleaned.push(' ');
        }
    }

    cleaned.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// Computes token-based Jaccard similarity between two strings (range 0.0 to 1.0).
pub fn token_similarity(a: &str, b: &str) -> f64 {
    let tokens_a: HashSet<&str> = a.split_whitespace().collect();
    let tokens_b: HashSet<&str> = b.split_whitespace().collect();

    if tokens_a.is_empty() && tokens_b.is_empty() {
        return 1.0;
    }
    if tokens_a.is_empty() || tokens_b.is_empty() {
        return 0.0;
    }

    let intersection_count = tokens_a.intersection(&tokens_b).count();
    let union_count = tokens_a.union(&tokens_b).count();

    if union_count == 0 {
        0.0
    } else {
        intersection_count as f64 / union_count as f64
    }
}

/// Computes the Levenshtein distance between two strings.
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let v1: Vec<char> = s1.chars().collect();
    let v2: Vec<char> = s2.chars().collect();

    let len1 = v1.len();
    let len2 = v2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut row: Vec<usize> = (0..=len2).collect();

    for (i, c1) in v1.iter().enumerate() {
        let mut prev_diag = row[0];
        row[0] = i + 1;

        for (j, c2) in v2.iter().enumerate() {
            let old_row_j = row[j + 1];
            let cost = if c1 == c2 { 0 } else { 1 };
            let current = (row[j] + 1)
                .min(row[j + 1] + 1)
                .min(prev_diag + cost);
            prev_diag = old_row_j;
            row[j + 1] = current;
        }
    }

    row[len2]
}

/// Normalized Levenshtein similarity (range 0.0 to 1.0).
pub fn levenshtein_similarity(s1: &str, s2: &str) -> f64 {
    let max_len = s1.chars().count().max(s2.chars().count());
    if max_len == 0 {
        return 1.0;
    }
    let dist = levenshtein_distance(s1, s2);
    1.0 - (dist as f64 / max_len as f64)
}

/// Calculates composite similarity score between two tracks (0.0 to 1.0).
pub fn calculate_match_score(target: &Track, candidate: &Track) -> f64 {
    let norm_target_title = normalize_string(&target.title);
    let norm_cand_title = normalize_string(&candidate.title);

    let norm_target_artist = normalize_string(&target.artist);
    let norm_cand_artist = normalize_string(&candidate.artist);

    let title_jaccard = token_similarity(&norm_target_title, &norm_cand_title);
    let title_lev = levenshtein_similarity(&norm_target_title, &norm_cand_title);
    let title_score = (title_jaccard * 0.5) + (title_lev * 0.5);

    let artist_jaccard = token_similarity(&norm_target_artist, &norm_cand_artist);
    let artist_lev = levenshtein_similarity(&norm_target_artist, &norm_cand_artist);
    let artist_score = (artist_jaccard * 0.5) + (artist_lev * 0.5);

    // Duration score: penalty if duration difference exceeds 5 seconds
    let duration_score = if target.duration > 0 && candidate.duration > 0 {
        let diff = if target.duration > candidate.duration {
            target.duration - candidate.duration
        } else {
            candidate.duration - target.duration
        };

        if diff <= 3 {
            1.0
        } else if diff <= 8 {
            0.8
        } else if diff <= 15 {
            0.5
        } else {
            0.2
        }
    } else {
        0.7 // neutral if duration unknown
    };

    // Composite weighted score: Title (50%), Artist (30%), Duration (20%)
    (title_score * 0.50) + (artist_score * 0.30) + (duration_score * 0.20)
}

/// Finds the best matching track candidate among a list.
pub fn find_best_match<'a>(target: &Track, candidates: &'a [Track], threshold: f64) -> Option<&'a Track> {
    candidates
        .iter()
        .map(|c| (c, calculate_match_score(target, c)))
        .filter(|(_, score)| *score >= threshold)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(track, _)| track)
}

/// Determines if two tracks are considered matching.
pub fn is_match(a: &Track, b: &Track) -> bool {
    calculate_match_score(a, b) >= 0.65
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let raw = "Midnight City (Official Music Video) [HD]";
        assert_eq!(normalize_string(raw), "midnight city");
    }

    #[test]
    fn test_similarity() {
        let s1 = normalize_string("M83 - Midnight City");
        let s2 = normalize_string("Midnight City by M83 (Audio)");
        assert!(token_similarity(&s1, &s2) > 0.6);
    }
}
