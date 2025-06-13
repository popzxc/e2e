use std::fmt;

pub(super) mod console;

use crate::{TestError, TestSuiteResult};

pub trait Reporter {
    fn name(&self) -> &'static str;
    fn on_test_suite_creation_started(&mut self, name: &str);
    fn on_test_suite_creation_finished(&mut self, name: &str, error: Option<&TestError>);
    fn on_test_suite_start(&mut self, name: &str);
    fn on_test_suite_end(&mut self, name: &str, result: &TestSuiteResult);
    fn on_test_start(&mut self, name: &str);
    fn on_test_ignored(&mut self, name: &str);
    fn on_test_end(&mut self, name: &str, error: Option<&TestError>);
}

impl fmt::Debug for dyn Reporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Reporter: {}", self.name())
    }
}
