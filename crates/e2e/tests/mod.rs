use e2e::test_suite;

#[derive(Debug, Clone)]
struct TestConfig;

#[derive(Debug, Clone)]
struct TestFlow(u32);

#[test_suite("My test suite")]
impl TestFlow {
    #[constructor]
    async fn new(_c: &TestConfig) -> anyhow::Result<Self> {
        Ok(Self(42))
    }

    #[test_case("Test case 1")]
    async fn test_case_1(&self) -> anyhow::Result<()> {
        assert_eq!(self.0, 42);
        Ok(())
    }

    #[test_case("Test case 2")]
    async fn test_case_2(&self) -> anyhow::Result<()> {
        assert!(self.0 > 0);
        Ok(())
    }
}

#[tokio::test]
async fn run_test() {
    let config = TestConfig;
    let mut tester = e2e::Tester::new(config);
    tester.add_suite(TestFlow::factory());
    tester.run().await.unwrap();
}
