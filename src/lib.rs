use std::cell::RefCell;

use log::Log;
use parking_lot::{ lock_api::ReentrantMutexGuard, RawMutex, RawThreadId, ReentrantMutex };

static MOCK_LOGGER: MockLogger = MockLogger::new();

pub struct MockLoggerGuard<'a>(
    ReentrantMutexGuard<'a, RawMutex, RawThreadId, RefCell<Option<Box<dyn Log>>>>,
);

impl<'a> Drop for MockLoggerGuard<'a> {
    fn drop(&mut self) {
        MockLogger::clear();
    }
}

pub struct MockLogger {
    mutex: ReentrantMutex<RefCell<Option<Box<dyn log::Log>>>>,
}

impl MockLogger {
    const fn new() -> Self {
        MockLogger { mutex: ReentrantMutex::new(RefCell::new(None)) }
    }

    pub fn set_logger<'a>(logger: impl log::Log + 'static) -> MockLoggerGuard<'a> {
        let lock = MOCK_LOGGER.mutex.lock();
        lock.borrow_mut().replace(Box::new(logger));

        let _ = log::set_logger(&MOCK_LOGGER);

        MockLoggerGuard(lock)
    }

    pub fn clear() {
        let lock = MOCK_LOGGER.mutex.lock();
        lock.borrow_mut().take();
    }
}

impl log::Log for MockLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        let lock = self.mutex.lock();
        let data = lock.borrow_mut();
        if let Some(logger) = data.as_ref() {
            return logger.enabled(metadata);
        }
        false
    }

    fn log(&self, record: &log::Record) {
        let lock = self.mutex.lock();
        let data = lock.borrow_mut();
        if let Some(logger) = data.as_ref() {
            logger.log(record);
        }
    }

    fn flush(&self) {
        let lock = self.mutex.lock();
        let data = lock.borrow_mut();
        if let Some(logger) = data.as_ref() {
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
    fn it_works() {
        let mut my_logger = MockMyLogger::new();
        my_logger.expect_log().once().return_const(());

        let _guard = MockLogger::set_logger(my_logger);
        log::set_max_level(log::LevelFilter::Trace);

        log::info!("ok");
    }

    #[test]
    fn it_works_2() {
        let mut my_logger = MockMyLogger::new();
        my_logger.expect_log().never().return_const(());

        let _guard = MockLogger::set_logger(my_logger);
        log::set_max_level(log::LevelFilter::Trace);

        // log::info!("ok");
    }
}
