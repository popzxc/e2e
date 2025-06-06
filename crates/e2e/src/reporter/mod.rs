use std::fmt;

pub(super) mod console;

use crate::TestError;

pub trait Reporter {
    fn name(&self) -> &'static str;
    fn on_test_suite_creation_started(&self, name: &str);
    fn on_test_suite_creation_finished(&self, name: &str);
    fn on_test_suite_start(&self, name: &str);
    fn on_test_suite_end(&self, name: &str);
    fn on_test_start(&self, name: &str);
    fn on_test_ignored(&self, name: &str);
    fn on_test_end(&self, name: &str);
    fn on_error(&self, err: &TestError);
}

impl fmt::Debug for dyn Reporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Reporter: {}", self.name())
    }
}
