# Integration Tests

This directory contains integration tests for the Laserfiche API client library.

## Test Environment Setup

To run the integration tests, you need to set the following environment variables:

```bash
export LF_TEST_API_ADDRESS="https://your-laserfiche-server.com"
export LF_TEST_REPOSITORY="your-repository-name"
export LF_TEST_USERNAME="your-test-username"
export LF_TEST_PASSWORD="your-test-password"
```

### Required Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `LF_TEST_API_ADDRESS` | The Laserfiche API server address | `https://api.laserfiche.com` |
| `LF_TEST_REPOSITORY` | The repository name to test against | `TestRepository` |
| `LF_TEST_USERNAME` | Username for authentication | `testuser` |
| `LF_TEST_PASSWORD` | Password for authentication | `********` |

## Running Tests

### Run all tests
```bash
cargo test
```

### Run integration tests only
```bash
cargo test --test integration_tests
```

### Run error handling tests
```bash
cargo test --test error_handling_tests
```

### Run with verbose output
```bash
cargo test -- --nocapture
```

## Test Structure

### Test Helpers (`test_helpers.rs`)

The test helpers module provides:
- `TestConfig`: A configuration struct that loads environment variables
- `skip_if_no_config!()`: A macro that skips tests when env vars are missing
- `assert_auth_success!()`: A macro for validating authentication responses
- Helper functions for validating entries and collections

### Integration Tests (`integration_tests.rs`)

Core functionality tests:
- Authentication flow
- Token refresh
- Blocking authentication
- Listing entries
- Getting specific entries
- Searching entries

### Error Handling Tests (`error_handling_tests.rs`)

Tests for error scenarios:
- Invalid credentials
- Nonexistent entries
- Invalid search filters
- Expired token refresh
- Expected panic scenarios

## Error Messages

All tests now provide clear, actionable error messages:

- **Missing configuration**: Tests will skip with detailed messages about which environment variables are missing
- **API errors**: Full error details including status codes and messages
- **Assertion failures**: Descriptive messages explaining what was expected vs. what was received

## Skipped Tests

When tests are skipped due to missing configuration, you'll see output like:

```
⚠️  Skipping test: Missing required environment variable: LF_TEST_API_ADDRESS
   To run this test, set the following environment variables:
   - LF_TEST_API_ADDRESS: The Laserfiche API server address
   - LF_TEST_REPOSITORY: The repository name
   - LF_TEST_USERNAME: Your test username
   - LF_TEST_PASSWORD: Your test password
```

## Mock Testing

For offline testing without a real Laserfiche server, consider:
1. Setting up mock HTTP responses using libraries like `mockito` or `httpmock`
2. Creating test fixtures with sample API responses
3. Using dependency injection to swap real clients with mock implementations

## Contributing

When adding new tests:
1. Use the `TestConfig` struct for configuration management
2. Use `expect()` with descriptive messages instead of `unwrap()`
3. Provide clear error messages for all assertions
4. Use the helper macros and functions to reduce duplication
5. Document any special setup requirements