// Panic recovery middleware for mesh relay handlers
pub fn handle_panic_safe<F, R>(f: F) -> Result<R, &'static str>
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    std::panic::catch_unwind(f).map_err(|_| "Handler panicked - recovered safely")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panic_recovery() {
        let res = handle_panic_safe(|| {
            panic!("test panic");
        });
        assert!(res.is_err());
    }
}
