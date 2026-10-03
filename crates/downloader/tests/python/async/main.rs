mod lrclib_tests;

#[pyo3_async_runtimes::tokio::main]
async fn main() -> pyo3::PyResult<()> {
    pyo3_async_runtimes::testing::main().await
}
