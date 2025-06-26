//! JSON Schema support for ISIN types via [schemars](https://crates.io/crates/schemars).
//!
//! This module provides JSON Schema generation that matches the serde deserialization behavior.
//! ISIN values are accepted as strings in the format of 12 characters: 2 uppercase letters
//! followed by 9 alphanumeric characters and 1 digit check digit.
//!
//! # Example
//!
//! ```rust
//! # #[cfg(feature = "schemars")]
//! # {
//! use isin::ISIN;
//! use schemars::schema_for;
//!
//! let schema = schema_for!(ISIN);
//! println!("{}", serde_json::to_string_pretty(&schema).unwrap());
//! # }
//! ```

use schemars::{
    schema::{InstanceType, Schema, SchemaObject, SingleOrVec, StringValidation},
    JsonSchema,
};

impl JsonSchema for crate::ISIN {
    fn schema_name() -> String {
        "ISIN".to_string()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> Schema {
        let schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
            string: Some(Box::new(StringValidation {
                pattern: Some("^[A-Z]{2}[0-9A-Z]{9}[0-9]$".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        schema.into()
    }
}

#[cfg(test)]
mod tests {
    use schemars::schema_for;

    #[test]
    fn schema_generation() {
        let schema = schema_for!(crate::ISIN);

        // Verify schema has correct title
        assert_eq!(
            schema
                .schema
                .metadata
                .as_ref()
                .unwrap()
                .title
                .as_ref()
                .unwrap(),
            "ISIN"
        );
    }

    #[test]
    fn schema_validation() {
        use jsonschema::Validator;
        use serde_json::json;

        let schema = schema_for!(crate::ISIN);
        let compiled_schema = Validator::new(&serde_json::to_value(schema).unwrap())
            .expect("Schema compilation failed");

        // Valid ISIN values should validate
        assert!(compiled_schema.is_valid(&json!("US0378331005"))); // Apple
        assert!(compiled_schema.is_valid(&json!("JP3788600009"))); // Hitachi
        assert!(compiled_schema.is_valid(&json!("IE00BFXC1P95"))); // Example from standard

        // Invalid values should not validate
        assert!(!compiled_schema.is_valid(&json!("US037833100"))); // Too short
        assert!(!compiled_schema.is_valid(&json!("US0378331005X"))); // Too long
        assert!(!compiled_schema.is_valid(&json!("us0378331005"))); // Lowercase prefix
        assert!(!compiled_schema.is_valid(&json!("US037833100A"))); // Invalid check digit (letter instead of digit)
        assert!(!compiled_schema.is_valid(&json!(123456789012i64))); // Number instead of string
        assert!(!compiled_schema.is_valid(&json!(""))); // Empty string
    }

    #[cfg(all(feature = "serde", feature = "schemars"))]
    #[test]
    fn schema_matches_serde() {
        use jsonschema::Validator;
        use serde_json::json;

        let schema = schema_for!(crate::ISIN);
        let compiled_schema = Validator::new(&serde_json::to_value(schema).unwrap())
            .expect("Schema compilation failed");

        // Test valid ISIN that serde can deserialize
        let valid_isin = "US0378331005";
        let parsed_isin = crate::parse(valid_isin).unwrap();
        let serialized = serde_json::to_value(parsed_isin).unwrap();

        // Schema should validate the serialized value
        assert!(compiled_schema.is_valid(&serialized));

        // Schema should also validate the raw string
        assert!(compiled_schema.is_valid(&json!(valid_isin)));
    }

    #[cfg(all(feature = "serde", feature = "schemars"))]
    #[test]
    fn struct_with_isin_roundtrip() {
        use jsonschema::Validator;
        use serde::{Deserialize, Serialize};
        use serde_json::json;

        #[derive(Serialize, Deserialize, schemars::JsonSchema)]
        struct SecurityInfo {
            isin: crate::ISIN,
            name: String,
        }

        let schema = schema_for!(SecurityInfo);
        let compiled_schema = Validator::new(&serde_json::to_value(schema).unwrap())
            .expect("Schema compilation failed");

        let test_data = json!({
            "isin": "US0378331005",
            "name": "Apple Inc."
        });

        // Should validate against schema
        assert!(compiled_schema.is_valid(&test_data));

        // Should deserialize successfully
        let security: SecurityInfo = serde_json::from_value(test_data).unwrap();
        assert_eq!(security.isin.to_string(), "US0378331005");
        assert_eq!(security.name, "Apple Inc.");
    }

    #[test]
    fn edge_case_patterns() {
        use jsonschema::Validator;
        use serde_json::json;

        let schema = schema_for!(crate::ISIN);
        let compiled_schema = Validator::new(&serde_json::to_value(schema).unwrap())
            .expect("Schema compilation failed");

        // Test various valid patterns from the ISIN standard
        assert!(compiled_schema.is_valid(&json!("XS2021448886"))); // International
        assert!(compiled_schema.is_valid(&json!("EZR9HY1361L7"))); // OTC derivative
        assert!(compiled_schema.is_valid(&json!("DE000A0GNPZ3"))); // German
        assert!(compiled_schema.is_valid(&json!("GB00BF0FCW58"))); // UK

        // Test boundary conditions
        assert!(!compiled_schema.is_valid(&json!("A20378331005"))); // Single letter prefix
        assert!(!compiled_schema.is_valid(&json!("US037833100A"))); // Letter as check digit
        assert!(!compiled_schema.is_valid(&json!("US037833-005"))); // Special character
    }
}
