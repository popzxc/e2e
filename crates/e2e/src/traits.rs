use std::fmt;

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

#[async_trait::async_trait]
pub trait Test: Send + Sync + 'static {
    fn name(&self) -> String;
    async fn run(&self) -> anyhow::Result<()>;
    fn ignore(&self) -> bool {
        false
    }

    fn only(&self) -> bool {
        false
    }
}

impl fmt::Debug for dyn Test {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
