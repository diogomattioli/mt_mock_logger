use std::{ collections::HashMap, sync::{ LazyLock, RwLock }, thread::ThreadId };

static MOCK_LOGGER: MockLogger = MockLogger::new();

pub struct MockLoggerGuard;

impl Drop for MockLoggerGuard {
    fn drop(&mut self) {
        MockLogger::remove_logger();
    }
}

pub struct MockLogger {
    mutex: LazyLock<RwLock<HashMap<ThreadId, (Box<dyn log::Log>, log::LevelFilter)>>>,
}

impl MockLogger {
    const fn new() -> Self {
        MockLogger {
            mutex: LazyLock::new(|| {
                let _ = log::set_logger(&MOCK_LOGGER);
                log::set_max_level(log::LevelFilter::Trace);
                RwLock::new(HashMap::new())
            }),
        }
    }

    pub fn set_logger(
        logger: impl log::Log + 'static,
        max_level: log::LevelFilter
    ) -> MockLoggerGuard {
        MOCK_LOGGER.mutex.write()
            .expect("mutex is poisoned")
            .insert(std::thread::current().id(), (Box::new(logger), max_level));

        MockLoggerGuard
    }

    fn remove_logger() {
        MOCK_LOGGER.mutex.write().expect("mutex is poisoned").remove(&std::thread::current().id());
    }
}

impl log::Log for MockLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        if
            let Some((logger, _)) = self.mutex
                .read()
                .expect("mutex is poisoned")
                .get(&std::thread::current().id())
        {
            return logger.enabled(metadata);
        }

        false
    }

    fn log(&self, record: &log::Record) {
        if
            let Some((logger, max_level)) = self.mutex
                .read()
                .expect("mutex is poisoned")
                .get(&std::thread::current().id())
        {
            if record.level() <= *max_level {
                logger.log(record);
            }
        }
    }

    fn flush(&self) {
        if
            let Some((logger, _)) = self.mutex
                .read()
                .expect("mutex is poisoned")
                .get(&std::thread::current().id())
        {
            logger.flush();
        }
    }
}

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
            .withf(|r| r.level() == log::LevelFilter::Info)
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
