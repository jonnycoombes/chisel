use chisel_common::char::coords::Coords;

/// Struct for holding some expected results for a [TestSpecification]
#[derive(Debug, Clone)]
pub struct TestExpectedResult {
    pub coords: Coords,
}

/// Struct for defining some test CASES
#[derive(Debug, Clone)]
pub struct TestSpecification {
    /// A filename to parse
    pub filename: String,
    /// Optional coordinates
    pub expected: TestExpectedResult,
}

impl TestSpecification {
    /// Create a new test case
    pub fn new(filename: &str, coords: Coords) -> Self {
        TestSpecification {
            filename: String::from(filename),
            expected: TestExpectedResult { coords },
        }
    }
}

/// Return a vec of invalid json test examples, combined with the coordinates within the files
/// that the errors occur (and where they should be identified as occurring)
pub(crate) fn invalid_json_specs() -> Vec<TestSpecification> {
    vec![
        TestSpecification::new(
            "fixtures/json/invalid/invalid_root_object.json",
            Coords {
                line: 1,
                column: 1,
                absolute: 1,
            },
        ),
        TestSpecification::new(
            "fixtures/json/invalid/invalid_bool_1.json",
            Coords {
                line: 2,
                column: 23,
                absolute: 1,
            },
        ),
        TestSpecification::new(
            "fixtures/json/invalid/invalid_bool_2.json",
            Coords {
                line: 3,
                column: 22,
                absolute: 1,
            },
        ),
        TestSpecification::new(
            "fixtures/json/invalid/invalid_bool_3.json",
            Coords {
                line: 3,
                column: 35,
                absolute: 1,
            },
        ),
        TestSpecification::new(
            "fixtures/json/invalid/invalid_array_1.json",
            Coords {
                line: 2,
                column: 22,
                absolute: 1,
            },
        ),
        TestSpecification::new(
            "fixtures/json/invalid/invalid_array_2.json",
            Coords {
                line: 2,
                column: 33,
                absolute: 1,
            },
        ),
        TestSpecification::new(
            "fixtures/json/invalid/missing-colon.json",
            Coords {
                line: 530,
                column: 5,
                absolute: 1,
            },
        ),
        TestSpecification::new(
            "fixtures/json/invalid/unterminated.json",
            Coords {
                line: 719,
                column: 24,
                absolute: 1,
            },
        ),
    ]
}
