use std::panic::AssertUnwindSafe;

pub use self::{
    config::TestRunnerConfiguration,
    reporter::{Reporter, console::ConsoleReporter},
    traits::{Test, TestSuite, TestSuiteFactory},
};
/// Procedural macro for defining test suites.
pub use e2e_macro::test_suite;
use futures::FutureExt;

mod config;
mod reporter;
mod traits;

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct TestResult {
    pub name: String,
    pub ignored: bool,
    pub error: Option<TestError>,
}

impl TestResult {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ignored: false,
            error: None,
        }
    }

    pub fn passed(&self) -> bool {
        self.error.is_none()
    }

    pub fn set_ignored(&mut self, ignored: bool) {
        self.ignored = ignored;
    }

    pub fn set_error(&mut self, error: TestError) {
        self.error = Some(error);
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct TestSuiteResult {
    pub name: String,
    pub passed: bool,
    pub tests: Vec<TestResult>,
    pub error: Option<TestError>,
}

impl TestSuiteResult {
    pub fn new(name: String) -> Self {
        Self {
            name,
            passed: true,
            tests: Vec::new(),
            error: None,
        }
    }

    pub fn add_test_result(&mut self, result: TestResult) {
        if !result.passed() {
            self.passed = false;
        }
        self.tests.push(result);
    }

    pub fn set_error(&mut self, error: TestError) {
        self.error = Some(error);
        self.passed = false;
    }
}

pub fn init() {
    // Set global panic hook to ignore all the panics
    // TODO: probably not the best idea long-term.
    std::panic::set_hook(Box::new(|_| {
        // Ignore panics in tests
    }));
}

#[derive(Debug)]
pub struct TestRunner<C: std::fmt::Debug + 'static> {
    /// Configuration for the test suites.
    config: C,
    /// Configuration for the test runner.
    runner_config: TestRunnerConfiguration,
    /// List of test suites to run.
    test_suites: Vec<Box<dyn TestSuiteFactory<C>>>,
    /// Reporter for test events.
    reporter: Box<dyn Reporter>,
    /// Results of test runs
    results: Vec<TestSuiteResult>,
}

impl<C: std::fmt::Debug + 'static> TestRunner<C> {
    pub fn new(config: C) -> Self {
        Self {
            config,
            runner_config: Default::default(),
            test_suites: Vec::new(),
            reporter: Box::new(ConsoleReporter::new()),
            results: Vec::new(),
        }
    }

    pub fn with_runner_config(mut self, config: TestRunnerConfiguration) -> Self {
        self.runner_config = config;
        self
    }

    pub fn add_suite(&mut self, factory: Box<dyn TestSuiteFactory<C>>) {
        self.test_suites.push(factory);
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        for factory in &std::mem::take(&mut self.test_suites) {
            let name = factory.name();
            if self
                .runner_config
                .test_suite_filter
                .as_ref()
                .is_some_and(|filter| !filter.is_match(&name))
            {
                self.reporter.on_test_suite_ignored(&name);
                continue;
            }

            let mut result = TestSuiteResult::new(name.clone());

            self.reporter.on_test_suite_creation_started(&name);
            let suite_result = factory
                .create_suite(&self.config)
                .await
                .map_err(TestError::CreateSuite);
            self.reporter
                .on_test_suite_creation_finished(&name, suite_result.as_ref().err());
            self.reporter.on_test_suite_start(&name);
            match suite_result {
                Ok(suite) => {
                    self.run_suite(suite, &mut result).await;
                }
                Err(err) => {
                    result.set_error(err);
                }
            }
            self.reporter.on_test_suite_end(&name, &result);

            let passed = result.passed;
            self.results.push(result);

            if !passed && self.runner_config.fail_fast {
                break;
            }
        }

        Ok(())
    }

    async fn run_test(
        &mut self,
        suite: &dyn TestSuite,
        test: &dyn Test,
        ignore: bool,
    ) -> TestResult {
        let mut test_result = TestResult::new(test.name());

        if ignore {
            test_result.set_ignored(true);
            self.reporter.on_test_ignored(&test.name());
            return test_result;
        }

        if let Err(err) = suite.before_each().await.map_err(TestError::BeforeEach) {
            test_result.set_error(err);
            return test_result;
        }

        self.reporter.on_test_start(&test.name());

        // Handle panics in gests
        let panic_handling_future =
            AssertUnwindSafe(test.run())
                .catch_unwind()
                .map(|res| match res {
                    Ok(res) => res,
                    Err(panic_err) => {
                        let err = if let Some(err) = panic_err.downcast_ref::<String>() {
                            anyhow::format_err!("Test panicked with message: {}", err)
                        } else if let Some(err) = panic_err.downcast_ref::<&str>() {
                            anyhow::format_err!("Test panicked with message: {}", err)
                        } else {
                            anyhow::format_err!("Test panicked with an unknown error type")
                        };
                        Err(err)
                    }
                });

        let test_future = tokio::time::timeout(self.runner_config.timeout(), panic_handling_future)
            .map(|res| match res {
                Ok(res) => res,
                Err(_) => Err(anyhow::format_err!(
                    "Test timed out after {:?}",
                    self.runner_config.timeout()
                )),
            });

        let test_run_result = test_future.await.map_err(TestError::Test);

        self.reporter
            .on_test_end(&test.name(), test_run_result.as_ref().err());

        if let Err(err) = test_run_result {
            test_result.set_error(err);
            // Do not run `after_each` if failing fast.
            if self.runner_config.fail_fast {
                return test_result;
            }
        }

        // TODO: do not override test error
        if let Err(err) = suite.after_each().await.map_err(TestError::AfterEach) {
            test_result.set_error(err);
        }

        test_result
    }

    async fn run_suite(&mut self, suite: Box<dyn TestSuite>, result: &mut TestSuiteResult) {
        if let Err(err) = suite.before_all().await.map_err(TestError::BeforeAll) {
            result.set_error(err);
            return;
        }

        // Check if at least one test has `only` set to true.
        let has_only = suite.tests().iter().any(|test| test.only());

        for test in suite.tests() {
            let mut ignore = test.ignore() && !self.runner_config.run_ignored;
            ignore |= has_only && !test.only();
            ignore |= self
                .runner_config
                .test_case_filter
                .as_ref()
                .is_some_and(|filter| !filter.is_match(&test.name()));

            let test_result = self.run_test(&*suite, &*test, ignore).await;
            let test_passed = test_result.passed();
            result.add_test_result(test_result);
            if !test_passed && self.runner_config.fail_fast {
                return;
            }
        }

        if let Err(err) = suite.after_all().await.map_err(TestError::AfterAll) {
            result.set_error(err);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TestError {
    #[error("Failed to create test suite: {0:?}")]
    CreateSuite(anyhow::Error),
    #[error("Failed to run 'before_all' for the test suite: {0:?}")]
    BeforeAll(anyhow::Error),
    #[error("Failed to run 'before_each' the test suite: {0:?}")]
    BeforeEach(anyhow::Error),
    #[error("Failed to run 'after_each' the test: {0:?}")]
    AfterEach(anyhow::Error),
    #[error("Failed to run 'after_all' the test: {0:?}")]
    AfterAll(anyhow::Error),
    #[error("Test failed: {0:?}")]
    Test(anyhow::Error),
}

/// Re-exports for procedural macros.
#[doc(hidden)]
pub mod __private_reexports {
    pub use async_trait::async_trait;
}
