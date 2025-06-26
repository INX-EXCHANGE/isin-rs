//! Example of generating JSON Schema for ISIN type

use isin::ISIN;
use schemars::schema_for;

fn main() {
    // Generate the JSON schema for ISIN
    let schema = schema_for!(ISIN);

    // Convert to JSON and print
    let schema_json = serde_json::to_string_pretty(&schema).unwrap();
    println!("JSON Schema for ISIN:");
    println!("{}", schema_json);
}
