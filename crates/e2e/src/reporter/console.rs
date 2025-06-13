use console::Term;

use crate::{TestError, TestSuiteResult, reporter::Reporter};

#[derive(Debug)]
pub struct ConsoleReporter {
    term: Term,
}

impl ConsoleReporter {
    pub fn new() -> Self {
        ConsoleReporter {
            term: Term::stdout(),
        }
    }
}

impl Default for ConsoleReporter {
    fn default() -> Self {
        ConsoleReporter::new()
    }
}

impl Reporter for ConsoleReporter {
    fn name(&self) -> &'static str {
        "ConsoleReporter"
    }

    fn on_test_suite_creation_started(&mut self, name: &str) {
        self.term
            .write_line(&format!("🔧  Creating test suite: {name}",))
            .unwrap();
    }

    fn on_test_suite_creation_finished(&mut self, name: &str, error: Option<&TestError>) {
        self.term.clear_last_lines(1).unwrap();
        match error {
            Some(err) => {
                self.term
                    .write_line(&format!(
                        "❌  Error creating test suite: {name} (error: {})",
                        err
                    ))
                    .unwrap();
            }
            None => {
                self.term
                    .write_line(&format!("✅  Test suite created: {name} "))
                    .unwrap();
            }
        }
    }

    fn on_test_suite_start(&mut self, name: &str) {
        self.term.clear_last_lines(1).unwrap();
        self.term
            .write_line(&format!("▶️  Starting test suite: {name} "))
            .unwrap();
    }

    fn on_test_suite_end(&mut self, name: &str, result: &TestSuiteResult) {
        if result.passed {
            self.term
                .write_line(&format!("✅ Test suite {name} passed",))
                .unwrap();
        } else {
            self.term
                .write_line(&format!("❌ Test suite {name} failed"))
                .unwrap();
            if let Some(error) = &result.error {
                self.term
                    .write_line(&format!("  - Error: {}", error))
                    .unwrap();
            }
            if result.tests.iter().any(|test| !test.passed()) {
                self.term.write_line("  - Failed tests:").unwrap();
                for test in &result.tests {
                    if let Some(err) = &test.error {
                        self.term
                            .write_line(&format!("    - {}: {}", test.name, err))
                            .unwrap();
                    }
                }
            }
        }
    }

    fn on_test_start(&mut self, name: &str) {
        self.term.write_line(&format!("  - ▶️  {name}",)).unwrap();
    }

    fn on_test_ignored(&mut self, name: &str) {
        self.term
            .write_line(&format!("  - ⏭️  {name} (ignored)",))
            .unwrap();
    }

    fn on_test_end(&mut self, name: &str, error: Option<&TestError>) {
        self.term.clear_last_lines(1).unwrap();
        match error {
            Some(err) => {
                self.term
                    .write_line(&format!("  - ❌ {name} (error: {})", err))
                    .unwrap();
            }
            None => {
                self.term.write_line(&format!("  - ✅ {name}",)).unwrap();
            }
        }
    }
}
