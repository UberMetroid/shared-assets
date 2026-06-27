//! Per-IP failed-attempt tracking with lockout.
//!
//! Used by all companion apps to throttle PIN-guessing attacks. After
//! `max_attempts` failed attempts, the client IP is locked out for
//! `lockout_duration`. Lockouts are stored in a process-wide map so
//! they survive across requests.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// Single attempt record.
#[derive(Debug, Clone)]
pub struct Attempt {
    pub count: u32,
    pub last_attempt: Instant,
}

fn login_attempts() -> &'static Mutex<HashMap<String, Attempt>> {
    static ATTEMPTS: OnceLock<Mutex<HashMap<String, Attempt>>> = OnceLock::new();
    ATTEMPTS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// True if the given IP is currently locked out.
///
/// Lockout lasts `lockout_duration`. Once expired, the entry is cleared
/// and the IP is allowed to try again.
#[must_use]
pub fn is_locked_out(ip: &str, max_attempts: u32, lockout_duration: std::time::Duration) -> bool {
    if let Ok(mut attempts) = login_attempts().lock()
        && let Some(attempt) = attempts.get(ip).cloned()
        && attempt.count >= max_attempts
    {
        if attempt.last_attempt.elapsed() < lockout_duration {
            return true;
        }
        attempts.remove(ip);
    }
    false
}

/// Record a failed attempt and return the updated record.
pub fn record_attempt(ip: &str) -> Attempt {
    if let Ok(mut attempts) = login_attempts().lock() {
        let now = Instant::now();
        let entry = attempts.entry(ip.to_string()).or_insert(Attempt {
            count: 0,
            last_attempt: now,
        });
        entry.count += 1;
        entry.last_attempt = now;
        entry.clone()
    } else {
        Attempt {
            count: 1,
            last_attempt: Instant::now(),
        }
    }
}

/// Clear the attempt record for the given IP (e.g. after a successful login).
pub fn reset_attempts(ip: &str) {
    if let Ok(mut attempts) = login_attempts().lock() {
        attempts.remove(ip);
    }
}

/// Seconds remaining in the lockout for the given IP, or 0 if not locked out.
#[must_use]
pub fn lockout_remaining_secs(ip: &str, lockout_duration: std::time::Duration) -> u64 {
    if let Ok(attempts) = login_attempts().lock()
        && let Some(attempt) = attempts.get(ip)
    {
        let elapsed = attempt.last_attempt.elapsed();
        if elapsed < lockout_duration {
            return (lockout_duration - elapsed).as_secs();
        }
    }
    0
}

/// Number of failed attempts currently recorded for the given IP.
///
/// Does not mutate state. Returns 0 if no attempts have been recorded or
/// if the lockout window has expired (the entry is cleared on read by
/// [`is_locked_out`]).
#[must_use]
pub fn current_attempts(ip: &str) -> u32 {
    if let Ok(attempts) = login_attempts().lock() {
        if let Some(attempt) = attempts.get(ip) {
            return attempt.count;
        }
    }
    0
}

/// Number of PIN attempts the IP has remaining before lockout.
///
/// Returns 0 when:
/// - the IP is currently locked out, or
/// - the IP has already reached `max_attempts` but the lockout window
///   has not yet expired.
///
/// Returns `max_attempts` when no failed attempts are recorded.
#[must_use]
pub fn attempts_left(ip: &str, max_attempts: u32, lockout_duration: std::time::Duration) -> u32 {
    if is_locked_out(ip, max_attempts, lockout_duration) {
        return 0;
    }
    let current = current_attempts(ip);
    if current >= max_attempts {
        0
    } else {
        max_attempts - current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset_for_test() {
        if let Ok(mut map) = login_attempts().lock() {
            map.clear();
        }
    }

    #[test]
    fn starts_unlocked() {
        reset_for_test();
        let lockout = std::time::Duration::from_secs(60);
        assert!(!is_locked_out("1.2.3.4", 3, lockout));
    }

    #[test]
    fn locks_after_max_attempts() {
        reset_for_test();
        let lockout = std::time::Duration::from_secs(60);
        for _ in 0..3 {
            record_attempt("1.2.3.4");
        }
        assert!(is_locked_out("1.2.3.4", 3, lockout));
    }

    #[test]
    fn not_locked_below_threshold() {
        reset_for_test();
        let lockout = std::time::Duration::from_secs(60);
        record_attempt("1.2.3.4");
        record_attempt("1.2.3.4");
        assert!(!is_locked_out("1.2.3.4", 3, lockout));
    }

    #[test]
    fn reset_clears_lockout() {
        reset_for_test();
        let lockout = std::time::Duration::from_secs(60);
        for _ in 0..3 {
            record_attempt("1.2.3.4");
        }
        assert!(is_locked_out("1.2.3.4", 3, lockout));
        reset_attempts("1.2.3.4");
        assert!(!is_locked_out("1.2.3.4", 3, lockout));
    }

    #[test]
    fn distinct_ips_are_independent() {
        reset_for_test();
        let lockout = std::time::Duration::from_secs(60);
        for _ in 0..3 {
            record_attempt("1.1.1.1");
        }
        assert!(is_locked_out("1.1.1.1", 3, lockout));
        assert!(!is_locked_out("2.2.2.2", 3, lockout));
    }

    #[test]
    fn remaining_secs_zero_when_not_locked() {
        reset_for_test();
        assert_eq!(
            lockout_remaining_secs("1.2.3.4", std::time::Duration::from_secs(60)),
            0
        );
    }

    #[test]
    fn remaining_secs_positive_when_locked() {
        reset_for_test();
        let lockout = std::time::Duration::from_secs(60);
        for _ in 0..3 {
            record_attempt("1.2.3.4");
        }
        let remaining = lockout_remaining_secs("1.2.3.4", lockout);
        assert!(remaining > 0 && remaining <= 60);
    }

    #[test]
    fn current_attempts_zero_when_unknown() {
        reset_for_test();
        assert_eq!(current_attempts("9.9.9.9"), 0);
    }

    #[test]
    fn current_attempts_reflects_recordings() {
        reset_for_test();
        record_attempt("9.9.9.9");
        record_attempt("9.9.9.9");
        assert_eq!(current_attempts("9.9.9.9"), 2);
    }

    #[test]
    fn current_attempts_cleared_after_reset() {
        reset_for_test();
        record_attempt("9.9.9.9");
        reset_attempts("9.9.9.9");
        assert_eq!(current_attempts("9.9.9.9"), 0);
    }

    #[test]
    fn attempts_left_max_when_no_attempts() {
        reset_for_test();
        let lockout = std::time::Duration::from_secs(60);
        assert_eq!(attempts_left("9.9.9.9", 5, lockout), 5);
    }

    #[test]
    fn attempts_left_decreases_with_failures() {
        reset_for_test();
        let lockout = std::time::Duration::from_secs(60);
        record_attempt("9.9.9.9");
        record_attempt("9.9.9.9");
        assert_eq!(attempts_left("9.9.9.9", 5, lockout), 3);
    }

    #[test]
    fn attempts_left_zero_when_locked() {
        reset_for_test();
        let lockout = std::time::Duration::from_secs(60);
        for _ in 0..5 {
            record_attempt("9.9.9.9");
        }
        assert_eq!(attempts_left("9.9.9.9", 5, lockout), 0);
    }
}
