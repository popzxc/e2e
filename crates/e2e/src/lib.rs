pub use self::{
    reporter::{Reporter, console::ConsoleReporter},
    traits::{Test, TestSuite, TestSuiteFactory},
};
/// Procedural macro for defining test suites.
pub use e2e_macro::test_suite;

mod reporter;
mod traits;

#[derive(Debug)]
pub struct Tester<C: std::fmt::Debug + 'static> {
    /// Configuration for the tester.
    config: C,
    /// List of test suites to run.
    test_suites: Vec<Box<dyn TestSuiteFactory<C>>>,
    /// Reporter for test events.
    reporter: Box<dyn Reporter>,
}

impl<C: std::fmt::Debug + 'static> Tester<C> {
    pub fn new(config: C) -> Self {
        Self {
            config,
            test_suites: Vec::new(),
            reporter: Box::new(ConsoleReporter::new()),
        }
    }

    pub fn add_suite(&mut self, factory: Box<dyn TestSuiteFactory<C>>) {
        self.test_suites.push(factory);
    }

    pub async fn run(self) -> Result<(), TestError> {
        for factory in &self.test_suites {
            let name = factory.name();
            self.reporter.on_test_suite_creation_started(&name);
            let suite_result = factory
                .create_suite(&self.config)
                .await
                .map_err(TestError::CreateSuite);
            self.reporter
                .on_test_suite_creation_finished(&name, suite_result.as_ref().err());
            let Ok(suite) = suite_result else {
                continue; // Skip this suite if creation failed
            };

            self.reporter.on_test_suite_start(&name);
            let suite_run_result = self.run_suite(suite).await;
            self.reporter
                .on_test_suite_end(&name, suite_run_result.as_ref().err());
        }

        Ok(())
    }

    async fn run_suite(&self, suite: Box<dyn TestSuite>) -> Result<(), TestError> {
        suite.before_all().await.map_err(TestError::BeforeAll)?;

        // Check if at least one test has `only` set to true.
        let has_only = suite.tests().iter().any(|test| test.only());

        for test in suite.tests() {
            if test.ignore() || (has_only && !test.only()) {
                self.reporter.on_test_ignored(&test.name());
                continue;
            }

            suite.before_each().await.map_err(TestError::BeforeEach)?;
            self.reporter.on_test_start(&test.name());
            let test_result = test
                .run()
                .await
                .map_err(|err| TestError::Test(test.name(), err));

            self.reporter
                .on_test_end(&test.name(), test_result.as_ref().err());

            suite.after_each().await.map_err(TestError::AfterEach)?;
        }

        suite.after_all().await.map_err(TestError::AfterAll)?;

        Ok(())
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
    #[error("Failed to run 'after_each' the test suite: {0:?}")]
    AfterEach(anyhow::Error),
    #[error("Failed to run 'after_all' the test suite: {0:?}")]
    AfterAll(anyhow::Error),
    #[error("Test {0} failed: {1:?}")]
    Test(String, anyhow::Error),
}

/// Re-exports for procedural macros.
#[doc(hidden)]
pub mod __private_reexports {
    pub use async_trait::async_trait;
}
