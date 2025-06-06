use console::Term;

use crate::reporter::Reporter;

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

    fn on_test_suite_creation_started(&self, name: &str) {
        self.term
            .write_line(&format!("🔧 Creating test suite: {name}",))
            .unwrap();
    }

    fn on_test_suite_creation_finished(&self, name: &str) {
        self.term.clear_last_lines(1).unwrap();
        self.term
            .write_line(&format!("✅ Test suite created: {name} "))
            .unwrap();
    }

    fn on_test_suite_start(&self, name: &str) {
        self.term.clear_last_lines(1).unwrap();
        self.term
            .write_line(&format!("▶️ Starting test suite: {name} "))
            .unwrap();
    }

    fn on_test_suite_end(&self, _name: &str) {
        // Nothing to report
    }

    fn on_test_start(&self, name: &str) {
        self.term.write_line(&format!("  - ▶️ {name}",)).unwrap();
    }

    fn on_test_ignored(&self, name: &str) {
        self.term
            .write_line(&format!("  - ⏭️  {name} (ignored)",))
            .unwrap();
    }

    fn on_test_end(&self, name: &str) {
        self.term.clear_last_lines(1).unwrap();
        self.term.write_line(&format!("  - ✅ {name}",)).unwrap();
    }

    fn on_error(&self, err: &crate::TestError) {
        self.term.clear_last_lines(1).unwrap();
        self.term.write_line(&format!("  - ❌ {}", err)).unwrap();
    }
}
