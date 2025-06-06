use e2e::test_suite;

#[derive(Debug, Clone)]
struct TestConfig {
    value: u32,
}

#[derive(Debug, Clone)]
struct TestFlow {
    value: u32,
}

#[test_suite("My test suite")]
impl TestFlow {
    #[constructor]
    async fn new(c: &TestConfig) -> anyhow::Result<Self> {
        Ok(Self { value: c.value })
    }

    #[before_each]
    async fn before_each(&self) -> anyhow::Result<()> {
        tracing::info!("before_each called");
        Ok(())
    }

    #[after_each]
    async fn after_each(&self) -> anyhow::Result<()> {
        tracing::info!("after_each called");
        Ok(())
    }

    #[before_all]
    async fn before_all(&self) -> anyhow::Result<()> {
        tracing::info!("before_all called");
        Ok(())
    }

    #[after_all]
    async fn after_all(&self) -> anyhow::Result<()> {
        tracing::info!("after_all called");
        Ok(())
    }

    #[test_case("Test case 1")]
    async fn test_case_1(&self) -> anyhow::Result<()> {
        assert_eq!(self.value, 42);
        Ok(())
    }

    #[test_case("Test case 2")]
    #[ignore]
    async fn test_case_2(&self) -> anyhow::Result<()> {
        assert!(self.value > 0);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = TestConfig { value: 42 };
    let mut tester = e2e::Tester::new(config);
    tester.add_suite(TestFlow::factory());
    tester.run().await?;
    Ok(())
}
