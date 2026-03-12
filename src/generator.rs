use dashmap::DashSet;
use rand::RngExt;
use rayon::prelude::*;
use std::collections::HashSet;

/// Every coupon code is exactly this many characters.
pub const COUPON_LENGTH: usize = 10;

/// The pool of allowed characters: uppercase A-Z and digits 0-9.
pub const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// If we've tried this many times the batch size and still don't have
/// enough unique coupons, something is wrong (prefix too long, etc).
pub const MAX_ATTEMPTS_MULTIPLIER: usize = 20;

/// Errors that can occur during generation.
#[derive(Debug, PartialEq)]
pub enum GeneratorError {
    /// The prefix is so long there's no room for random characters.
    PrefixTooLong,
    /// The caller asked for zero coupons.
    ZeroCount,
    /// Couldn't generate enough unique coupons within the attempt limit.
    MaxAttemptsExceeded,
}

fn suffix_capacity(random_len: usize) -> u128 {
    let mut capacity = 1u128;

    for _ in 0..random_len {
        capacity = capacity.saturating_mul(CHARSET.len() as u128);
    }

    capacity
}

fn encode_suffix(mut index: u128, random_len: usize) -> String {
    let mut suffix = vec![CHARSET[0]; random_len];
    let base = CHARSET.len() as u128;

    for position in (0..random_len).rev() {
        let digit = (index % base) as usize;
        suffix[position] = CHARSET[digit];
        index /= base;
    }

    String::from_utf8(suffix).expect("CHARSET only contains ASCII characters")
}

fn sample_unique_suffix_indices(space_size: u128, count: usize) -> Vec<u128> {
    let mut rng = rand::rng();
    let mut selected = HashSet::with_capacity(count);
    let start = space_size - count as u128;

    for current in start..space_size {
        let candidate = rng.random_range(0..=current);
        if !selected.insert(candidate) {
            selected.insert(current);
        }
    }

    selected.into_iter().collect()
}

fn generate_near_capacity_coupons(
    prefix_upper: &str,
    random_len: usize,
    count: usize,
    suffix_space: u128,
) -> Vec<String> {
    let suffix_indices = sample_unique_suffix_indices(suffix_space, count);

    suffix_indices
        .into_par_iter()
        .map(|index| format!("{}{}", prefix_upper, encode_suffix(index, random_len)))
        .collect()
}

/// Generates `count` unique coupon codes with the given `prefix`.
///
/// - Total coupon length is always `COUPON_LENGTH` (10).
/// - The prefix is converted to uppercase.
/// - The random portion uses characters from `CHARSET` (a-z, 0-9).
/// - Generation is parallelized using `rayon`.
/// - Uniqueness is enforced using a concurrent `DashSet`.
///
/// # Errors
/// - `PrefixTooLong` if `prefix.len() >= COUPON_LENGTH`
/// - `ZeroCount` if `count == 0`
/// - `MaxAttemptsExceeded` if unique generation fails (extremely unlikely)
pub fn generate_coupons(
    prefix: &str,
    count: usize,
) -> Result<Vec<String>, GeneratorError> {
    // ── Validation ──────────────────────────────────────────
    let prefix_upper = prefix.to_ascii_uppercase();

    if prefix_upper.len() >= COUPON_LENGTH {
        return Err(GeneratorError::PrefixTooLong);
    }
    if count == 0 {
        return Err(GeneratorError::ZeroCount);
    }

    // ── Setup ───────────────────────────────────────────────
    let random_len = COUPON_LENGTH - prefix_upper.len();
    let suffix_space = suffix_capacity(random_len);

    if count as u128 > suffix_space {
        return Err(GeneratorError::MaxAttemptsExceeded);
    }

    if (count as u128).saturating_mul(2) >= suffix_space {
        return Ok(generate_near_capacity_coupons(
            &prefix_upper,
            random_len,
            count,
            suffix_space,
        ));
    }

    let set: DashSet<String> = DashSet::new();
    let max_total_attempts = count * MAX_ATTEMPTS_MULTIPLIER;
    let batch_size = count.max(1_024);
    let mut total_attempts: usize = 0;

    // ── Parallel Generation Loop ────────────────────────────
    while set.len() < count {
        if total_attempts > max_total_attempts {
            return Err(GeneratorError::MaxAttemptsExceeded);
        }

        let needed = count - set.len();
        let attempts_this_round = batch_size.min(needed * 2);

        // Generate candidates in parallel.
        (0..attempts_this_round)
            .into_par_iter()
            .for_each(|_| {
                // Early exit: stop generating if we already have enough.
                if set.len() >= count {
                    return;
                }

                // rand 0.10 syntax
                let mut rng = rand::rng();

                // Build the random part character by character.
                let random_part: String = (0..random_len)
                    .map(|_| {
                        // rand 0.10 uses random_range via RngExt
                        let idx = rng.random_range(0..CHARSET.len());
                        CHARSET[idx] as char
                    })
                    .collect();

                let coupon = format!("{}{}", prefix_upper, random_part);

                set.insert(coupon);
            });

        total_attempts += attempts_this_round;
    }

    // ── Collect Results ─────────────────────────────────────
    let result: Vec<String> = set.into_iter().take(count).collect();
    Ok(result)
}
