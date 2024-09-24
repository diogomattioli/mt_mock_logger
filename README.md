# mt-mock-logger

A thread-safe mock logger designed to support logger mocking across multiple tests running in parallel.

The injected logger is automatically scoped to the lifetime of the returned `MockLoggerGuard` provided by the `set_logger` function.

The `MockLogger` is not intended for use with async. Since it binds the injected logger to the test's `ThreadId`, using it in an async context may result in unexpected behavior.

# Usage

```rust
#[cfg(test)]
mod tests {
    use mockall::mock;

    use super::*;

    mock! {
        pub MyLogger {}
        impl log::Log for MyLogger {
            fn enabled<'a>(&self, metadata: &log::Metadata<'a>) -> bool;
            fn log<'a>(&self, record: &log::Record<'a>);
            fn flush(&self);
        }
    }

    #[test]
    fn test_logging() {
        let mut my_logger = MockMyLogger::new();
        my_logger
            .expect_log()
            .withf(|r| r.level() == log::LevelFilter::Info && 
                        r.args().as_str() == Some("ok"))
            .once()
            .return_const(());

        let _guard = MockLogger::set_logger(my_logger, log::LevelFilter::Info);

        log::info!("ok");
        log::trace!("ok");
    }

    #[test]
    fn test_logging_below_max_level() {
        let mut my_logger = MockMyLogger::new();
        my_logger.expect_log().never().return_const(());

        let _guard = MockLogger::set_logger(my_logger, log::LevelFilter::Info);

        log::trace!("ok");
    }

    #[test]
    fn test_no_logger() {
        log::trace!("ok");
    }
}
```