use std::time::Duration;

#[derive(Debug, Default, Clone, clap::Args)]
pub struct TestRunnerConfiguration {
    /// Regex filter for test suites.
    #[clap(long)]
    pub(crate) test_suite_filter: Option<regex::Regex>,
    /// Regex filter for test cases.
    #[clap(long)]
    pub(crate) test_case_filter: Option<regex::Regex>,
    /// Whether to run ignored tests.
    #[clap(long, default_value = "false")]
    pub(crate) run_ignored: bool,
    /// Timeout for each test case.
    #[clap(long)]
    pub(crate) timeout_ms: Option<u64>,
    /// Stop after the first failed test.
    #[clap(long)]
    pub(crate) fail_fast: bool,
}

impl TestRunnerConfiguration {
    const DEFAULT_TIMEOUT_MS: u64 = 60_000; // 60 seconds

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

    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms.unwrap_or(Self::DEFAULT_TIMEOUT_MS))
    }

    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }
}
