# Credits 
A huge thank you! to [dtolnay](https://github.com/dtolnay) for providing the [proc-macro-workshop](https://github.com/dtolnay/proc-macro-workshop). Without it
I'm not sure I could've learned rust's proc macros so fast.

# About this repository
This repository is my own solution to the dtolnay's proc-macro-workshop of the [Builder Macro](https://github.com/dtolnay/proc-macro-workshop?tab=readme-ov-file#derive-macro-derivebuilder).
As a beginner Rustacean, completing this workshop was a significant milestone. I am sharing this solution to document my progress and celebrate passing all 9 original test cases. (plus an additional edge case I identified)

As mentioned above, I added a 10th test case as it was a very low hanging fruit, which handles returning an error for incorrect field
attributes that are not assignments e.g. #[builder(eac)] should fail too. The resulting error's output can be found [here](tests/10-invalid-attribute-assignment.stderr).

# How to run the tests
```bash 
cargo test
```

# Test Results

| Test Case | Expectation | Result |
| :--- | :--- | :--- |
| `01-parse.rs` | [should pass] | **ok** ✅ |
| `02-create-builder.rs` | [should pass] | **ok** ✅ |
| `03-call-setters.rs` | [should pass] | **ok** ✅ |
| `04-call-build.rs` | [should pass] | **ok** ✅ |
| `05-method-chaining.rs` | [should pass] | **ok** ✅ |
| `06-optional-field.rs` | [should pass] | **ok** ✅ |
| `07-repeated-field.rs` | [should pass] | **ok** ✅ |
| `08-unrecognized-attribute.rs` | [should fail to compile] | **ok** ✅ |
| `09-redefined-prelude-types.rs` | [should pass] | **ok** ✅ |
| `10-invalid-attribute-assignment.rs` | [should fail to compile] | **ok** ✅ |
