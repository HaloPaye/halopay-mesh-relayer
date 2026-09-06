//! Panic recovery middleware for asynchronous mesh gossip workers.

#[derive(Debug, PartialEq, Eq)]
pub enum RecoveryResult<T> {
    Ok(T),
    Panicked(String),
}

pub fn handle_panic_safe<F, R>(f: F) -> RecoveryResult<R>
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(f) {
        Ok(val) => RecoveryResult::Ok(val),
        Err(err) => {
            let msg = if let Some(s) = err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = err.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic payload".to_string()
            };
            RecoveryResult::Panicked(msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panic_safe_success() {
        let res = handle_panic_safe(|| 42 * 2);
        assert_eq!(res, RecoveryResult::Ok(84));
    }

    #[test]
    fn test_panic_safe_caught() {
        let res = handle_panic_safe(|| {
            panic!("fatal worker panic");
        });
        assert_eq!(res, RecoveryResult::Panicked("fatal worker panic".to_string()));
    }
}
