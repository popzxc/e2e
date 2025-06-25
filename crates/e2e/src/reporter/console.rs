use console::Term;

use crate::{TestError, TestSuiteResult, reporter::Reporter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestStateMarker {
    Running,
    Ignored,
    Error,
    Success,
}

impl TestStateMarker {
    pub fn emoji(&self) -> &'static str {
        match self {
            TestStateMarker::Running => "▶️",
            TestStateMarker::Ignored => "⏭️",
            TestStateMarker::Error => "❌",
            TestStateMarker::Success => "✅",
        }
    }
}

#[derive(Debug)]
struct TestState {
    marker: TestStateMarker,
    name: String,
    error: Option<String>, // TODO: Should not be string
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestSuiteStateMarker {
    Creating,
    Running,
    Ignored,
    Error,
    Success,
}

impl TestSuiteStateMarker {
    pub fn emoji(&self) -> &'static str {
        match self {
            TestSuiteStateMarker::Creating => "🔧",
            TestSuiteStateMarker::Running => "▶️",
            TestSuiteStateMarker::Ignored => "⏭️",
            TestSuiteStateMarker::Error => "❌",
            TestSuiteStateMarker::Success => "✅",
        }
    }
}

#[derive(Debug)]
struct TestSuiteState {
    name: String,
    marker: TestSuiteStateMarker,
    tests: Vec<TestState>,
    error: Option<String>, // TODO: Should not be string
}

#[derive(Debug)]
pub struct ConsoleReporter {
    term: Term,
    lines_written: usize,
    suites: Vec<TestSuiteState>,
}

impl ConsoleReporter {
    pub fn new() -> Self {
        ConsoleReporter {
            term: Term::stdout(),
            lines_written: 0,
            suites: Vec::new(),
        }
    }

    pub fn write(&mut self) {
        self.term.clear_last_lines(self.lines_written).unwrap();
        let mut lines = Vec::new();
        for suite in &self.suites {
            let marker = suite.marker.emoji();
            lines.push(format!("{} Test Suite: {}", marker, suite.name));
            if let Some(error) = &suite.error {
                lines.push("  - Error:".to_string());
                for line in error.lines() {
                    lines.push(format!("    - {}", line));
                }
            }
            for test in &suite.tests {
                let test_marker = test.marker.emoji();
                let test_name = &test.name;
                if let Some(err) = &test.error {
                    lines.push(format!("  - {} {} error:", test_marker, test_name));
                    for line in err.lines() {
                        lines.push(format!("    | {}", line));
                    }
                } else {
                    lines.push(format!("  - {} {}", test_marker, test_name));
                }
            }
        }
        self.lines_written = lines.len();
        for line in lines {
            self.term.write_line(&line).unwrap();
        }
    }

    fn add_test_suite(&mut self, name: &str, marker: TestSuiteStateMarker) {
        let state = TestSuiteState {
            name: name.to_string(),
            marker,
            tests: Vec::new(),
            error: None,
        };
        self.suites.push(state);
    }

    fn update_test_suite(
        &mut self,
        name: &str,
        marker: TestSuiteStateMarker,
        error: Option<String>, // TODO: should not be string
    ) {
        if let Some(suite) = self.suites.iter_mut().find(|s| s.name == name) {
            suite.marker = marker;
            suite.error = error;
        }
    }

    fn add_test(&mut self, name: &str, marker: TestStateMarker) {
        if let Some(suite) = self.suites.last_mut() {
            let state = TestState {
                marker,
                name: name.to_string(),
                error: None,
            };
            suite.tests.push(state);
        } else {
            eprintln!("No test suite found to add test: {}", name);
        }
    }

    fn update_test(&mut self, name: &str, marker: TestStateMarker, error: Option<String>)
    // TODO: should not be string
    {
        if let Some(suite) = self.suites.last_mut() {
            if let Some(test) = suite.tests.iter_mut().find(|t| t.name == name) {
                test.marker = marker;
                test.error = error;
            } else {
                eprintln!("No test found to update: {}", name);
            }
        } else {
            eprintln!("No test suite found to update test: {}", name);
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
        self.add_test_suite(name, TestSuiteStateMarker::Creating);
        self.write();
    }

    fn on_test_suite_ignored(&mut self, name: &str) {
        self.add_test_suite(name, TestSuiteStateMarker::Ignored);
        self.write();
    }

    fn on_test_suite_creation_finished(&mut self, name: &str, error: Option<&TestError>) {
        if let Some(err) = error {
            self.update_test_suite(name, TestSuiteStateMarker::Error, Some(err.to_string()));
        }
        self.write();
    }

    fn on_test_suite_start(&mut self, name: &str) {
        self.update_test_suite(name, TestSuiteStateMarker::Running, None);
        self.write();
    }

    fn on_test_suite_end(&mut self, name: &str, result: &TestSuiteResult) {
        self.update_test_suite(
            name,
            if result.error.is_some() {
                TestSuiteStateMarker::Error
            } else {
                TestSuiteStateMarker::Success
            },
            result.error.as_ref().map(|e| e.to_string()),
        );
        self.write();
        // TODO: probably we can remove this suite from the list smth like
        // self.suites.retain(|s| s.name != name);
    }

    fn on_test_start(&mut self, name: &str) {
        self.add_test(name, TestStateMarker::Running);
        self.write();
    }

    fn on_test_ignored(&mut self, name: &str) {
        self.add_test(name, TestStateMarker::Ignored);
        self.write();
    }

    fn on_test_end(&mut self, name: &str, error: Option<&TestError>) {
        self.update_test(
            name,
            if error.is_some() {
                TestStateMarker::Error
            } else {
                TestStateMarker::Success
            },
            error.as_ref().map(|e| e.to_string()),
        );
    }
}
