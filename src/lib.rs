//! ## convert json object to protobuf message
//!
//! json2pb is a library for converting json object to protobuf message.
//! It also provides a commandline program `j2pb`. For more information, run `j2pb -h`
//!
//! `j2pb -f test.json` this can convert test.json to pb message and print the result on the screen(stdout)
//!
//! `j2pb -f test.json -o test.proto` does the same thing but save the result in test.proto
//!
//! ## json2pb library
//!
//! And you can use json2pb in your project just by adding `json2pb="*"` in your cargo dependency.
//!
//! ### example
//! ```rust
//! use json2pb::parser;
//! use json2pb::pbgen;
//! use nom::error::VerboseError;
//!
//! let json_code = r#"
//!     {
//!         "name": "deen",
//!         "age": 26
//!     }
//! "#;
//!
//! let (_, json_value) = parser::root::<VerboseError<&str>>(json_code).unwrap();
//! let json_2_pb_ast = pbgen::visit_json_root(&json_value).unwrap();
//! let generated_pb_message = pbgen::gen_pb_def(&json_2_pb_ast);
//! assert_eq!(generated_pb_message, r#"message root_data {
//!     string name = 1;
//!     int64 age = 2;
//! }
//! "#);
//! ```


pub mod parser;
pub mod pbgen;

/// Common Result
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;