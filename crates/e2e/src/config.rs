use std::time::Duration;

#[derive(Debug, Default, Clone, clap::Args)]
pub struct TestRunnerConfiguration {
    /// Regex filter for test suites.
    #[clap(long)]
    test_suite_filter: Option<regex::Regex>,
    /// Regex filter for test cases.
    #[clap(long)]
    test_case_filter: Option<regex::Regex>,
    /// Whether to run ignored tests.
    #[clap(long, default_value = "false")]
    run_ignored: bool,
    /// Timeout for each test case.
    #[clap(long)]
    timeout_ms: Option<u64>,
}

impl TestRunnerConfiguration {
    pub fn with_test_suite_filter(mut self, filter: regex::Regex) -> Self {
        self.test_suite_filter = Some(filter);
        self
    }

    pub fn with_test_case_filter(mut self, filter: regex::Regex) -> Self {
        self.test_case_filter = Some(filter);
        self
    }

    pub fn with_run_ignored(mut self, run_ignored: bool) -> Self {
        self.run_ignored = run_ignored;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_ms = Some(timeout.as_millis() as u64);
        self
    }

    pub fn timeout(&self) -> Option<std::time::Duration> {
        self.timeout_ms.map(std::time::Duration::from_millis)
    }
}
