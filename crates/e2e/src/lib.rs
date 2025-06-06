use std::fmt;

pub use self::reporter::{Reporter, console::ConsoleReporter};
/// Procedural macro for defining test suites.
pub use e2e_macro::test_suite;

mod reporter;

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
            let suite = match factory
                .create_suite(&self.config)
                .await
                .map_err(TestError::CreateSuite)
            {
                Ok(suite) => suite,
                Err(err) => {
                    self.reporter.on_error(&err);
                    continue;
                }
            };
            self.reporter.on_test_suite_creation_finished(&name);

            self.reporter.on_test_suite_start(&name);
            if let Err(err) = self.run_suite(suite).await {
                self.reporter.on_error(&err);
            }
            self.reporter.on_test_suite_end(&name);
        }

        Ok(())
    }

    async fn run_suite(&self, suite: Box<dyn TestSuite>) -> Result<(), TestError> {
        suite.before_all().await.map_err(TestError::BeforeAll)?;

        for test in suite.tests() {
            if test.ignore() {
                self.reporter.on_test_ignored(&test.name());
                continue;
            }

            suite.before_each().await.map_err(TestError::BeforeEach)?;
            self.reporter.on_test_start(&test.name());
            if let Err(err) = test
                .run()
                .await
                .map_err(|err| TestError::Test(test.name(), err))
            {
                self.reporter.on_error(&err);
            } else {
                self.reporter.on_test_end(&test.name());
            }
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

#[async_trait::async_trait]
pub trait TestSuiteFactory<C>: Send + Sync + 'static {
    fn name(&self) -> String;

    /// Creates a new test suite instance.
    async fn create_suite(&self, config: &C) -> anyhow::Result<Box<dyn TestSuite>>;
}

impl<C: std::fmt::Debug + 'static> fmt::Debug for Box<dyn TestSuiteFactory<C>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[async_trait::async_trait]
pub trait TestSuite: Send + Sync + 'static {
    fn name(&self) -> String;

    fn tests(&self) -> Vec<Box<dyn Test>>;

    async fn before_all(&self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn before_each(&self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn after_each(&self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn after_all(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl fmt::Debug for dyn TestSuite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[async_trait::async_trait]
pub trait Test: Send + Sync + 'static {
    fn name(&self) -> String;
    async fn run(&self) -> anyhow::Result<()>;
    fn ignore(&self) -> bool {
        false
    }
}

impl fmt::Debug for dyn Test {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Re-exports for procedural macros.
#[doc(hidden)]
pub mod __private_reexports {
    pub use async_trait::async_trait;
}
