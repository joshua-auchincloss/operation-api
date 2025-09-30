use operation_api_sdk::operation;

#[operation(version = 1, describe(text = "sums values in a list of integers"))]
fn sum(values: Vec<i32>) -> i32 {
    values.iter().sum()
}

#[operation(version = 1)]
/// this is a test description
fn with_lt<'a>(value: &'a str) -> &'a str {
    value
}

#[test]
fn test() {
    sum(vec![1, 2, 3]);
    with_lt("abc");
}
