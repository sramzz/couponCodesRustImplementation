use coupon_generator::generator::{generate_coupons, GeneratorError, COUPON_LENGTH};
use std::collections::HashSet;

// ───────────────────────────────────────────────
// HAPPY PATH TESTS
// ───────────────────────────────────────────────

#[test]
fn returns_the_exact_number_of_coupons_requested() {
    let coupons = generate_coupons("SAN", 100).unwrap();
    assert_eq!(coupons.len(), 100);
}

#[test]
fn every_coupon_has_exactly_10_characters() {
    let coupons = generate_coupons("SAN", 200).unwrap();
    for coupon in &coupons {
        assert_eq!(
            coupon.len(),
            COUPON_LENGTH,
            "Coupon '{}' has wrong length: {}",
            coupon,
            coupon.len()
        );
    }
}

#[test]
fn every_coupon_starts_with_the_prefix_in_uppercase() {
    let coupons = generate_coupons("ABC", 50).unwrap();
    for coupon in &coupons {
        assert!(
            coupon.starts_with("ABC"),
            "Coupon '{}' does not start with 'ABC'",
            coupon
        );
    }
}

#[test]
fn all_coupons_in_a_batch_are_unique() {
    let coupons = generate_coupons("SAN", 5_000).unwrap();
    let unique: HashSet<&String> = coupons.iter().collect();
    assert_eq!(
        unique.len(),
        coupons.len(),
        "Found {} duplicates in {} coupons",
        coupons.len() - unique.len(),
        coupons.len()
    );
}

#[test]
fn coupons_contain_only_uppercase_letters_and_digits() {
    let coupons = generate_coupons("SAN", 500).unwrap();
    for coupon in &coupons {
        for ch in coupon.chars() {
            assert!(
                ch.is_ascii_uppercase() || ch.is_ascii_digit(),
                "Coupon '{}' contains invalid character '{}'",
                coupon,
                ch
            );
        }
    }
}

#[test]
fn prefix_is_treated_as_case_insensitive() {
    // "SAN", "san", "San" should all produce coupons starting with "SAN"
    for prefix in &["SAN", "san", "San", "sAn"] {
        let coupons = generate_coupons(prefix, 10).unwrap();
        for coupon in &coupons {
            assert!(
                coupon.starts_with("SAN"),
                "Prefix '{}' produced coupon '{}' — expected 'SAN' prefix",
                prefix,
                coupon
            );
        }
    }
}

#[test]
fn empty_prefix_generates_fully_random_coupons() {
    let coupons = generate_coupons("", 100).unwrap();
    assert_eq!(coupons.len(), 100);
    for coupon in &coupons {
        assert_eq!(coupon.len(), COUPON_LENGTH);
    }
}

#[test]
fn single_character_prefix_works() {
    let coupons = generate_coupons("X", 50).unwrap();
    for coupon in &coupons {
        assert!(coupon.starts_with("X"));
        assert_eq!(coupon.len(), COUPON_LENGTH);
    }
}

#[test]
fn nine_character_prefix_leaves_one_random_character() {
    // Prefix "ABCDEFGHI" = 9 chars, leaving 1 random char.
    // Only 36 possible unique coupons (A-Z, 0-9).
    let coupons = generate_coupons("ABCDEFGHI", 30).unwrap();
    assert_eq!(coupons.len(), 30);
    for coupon in &coupons {
        assert!(coupon.starts_with("ABCDEFGHI"));
        assert_eq!(coupon.len(), COUPON_LENGTH);
    }
}

#[test]
fn nine_character_prefix_can_generate_all_36_possible_codes() {
    let coupons = generate_coupons("ABCDEFGHI", 36).unwrap();
    assert_eq!(coupons.len(), 36);

    let unique: HashSet<&String> = coupons.iter().collect();
    assert_eq!(unique.len(), 36);

    for coupon in &coupons {
        assert!(coupon.starts_with("ABCDEFGHI"));
        assert_eq!(coupon.len(), COUPON_LENGTH);
    }
}

// ───────────────────────────────────────────────
// ERROR CASE TESTS
// ───────────────────────────────────────────────

#[test]
fn prefix_of_10_chars_returns_prefix_too_long_error() {
    // 10-char prefix leaves 0 random chars — impossible
    let result = generate_coupons("ABCDEFGHIJ", 10);
    assert_eq!(result, Err(GeneratorError::PrefixTooLong));
}

#[test]
fn prefix_longer_than_10_chars_returns_prefix_too_long_error() {
    let result = generate_coupons("ABCDEFGHIJKLM", 10);
    assert_eq!(result, Err(GeneratorError::PrefixTooLong));
}

#[test]
fn zero_count_returns_zero_count_error() {
    let result = generate_coupons("SAN", 0);
    assert_eq!(result, Err(GeneratorError::ZeroCount));
}

#[test]
fn requesting_more_than_36_codes_with_nine_character_prefix_returns_max_attempts_exceeded() {
    let result = generate_coupons("ABCDEFGHI", 37);
    assert_eq!(result, Err(GeneratorError::MaxAttemptsExceeded));
}

// ───────────────────────────────────────────────
// PERFORMANCE TEST
// ───────────────────────────────────────────────

#[test]
fn generates_50000_unique_coupons_without_crashing() {
    let coupons = generate_coupons("X", 50_000).unwrap();
    assert_eq!(coupons.len(), 50_000);
    let unique: HashSet<&String> = coupons.iter().collect();
    assert_eq!(unique.len(), 50_000);
}

#[test]
fn generates_10_000_000_unique_coupons_without_crashing() {
    // Note: this test can take a few seconds in release mode.
    let count = 10_000_000;
    let coupons = generate_coupons("", count).unwrap();
    assert_eq!(coupons.len(), count);
}
