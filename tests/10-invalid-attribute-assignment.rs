// I added this additional test as an extension for test suit 08-unrecognized-attribute
// to ensure that only only typos like #[builder(eac = "arg")] get a compile time error,
// but also non assignment statements like #[builder(eac)] are treated as an error.

use derive_builder::Builder;

#[derive(Builder)]
pub struct Command {
    executable: String,
    #[builder(eac)]
    args: Vec<String>,
    env: Vec<String>,
    current_dir: Option<String>,
}

fn main() {}
