@echo off
echo To run this test: ./fail FAILING_TEST_NUMBER
echo To run a specific test: ./fail TEST_NUMBER
cargo run --bin debashc -- fail %*
