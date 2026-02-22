use dsntk_feel::{scope, FeelScope};

#[test]
fn _0001() {
  let node = dsntk_feel_parser::parse_range_literal(&scope!(), "[1..100)", false).unwrap();
  let expected = r#"
       Range
       ├─ IntervalStart (closed)
       │  └─ Numeric
       │     └─ `1`
       └─ IntervalEnd (opened)
          └─ Numeric
             └─ `100`
    "#;
  assert_eq!(expected, node.trace());
}

#[test]
fn _0002() {
  let node = dsntk_feel_parser::parse_range_literal(&scope!(), "[0..14000]", false).unwrap();
  let expected = r#"
       Range
       ├─ IntervalStart (closed)
       │  └─ Numeric
       │     └─ `0`
       └─ IntervalEnd (closed)
          └─ Numeric
             └─ `14000`
    "#;
  assert_eq!(expected, node.trace());
}

#[test]
fn _0003() {
  let node = dsntk_feel_parser::parse_range_literal(&scope!(), "(14000..48000]", false).unwrap();
  let expected = r#"
       Range
       ├─ IntervalStart (opened)
       │  └─ Numeric
       │     └─ `14000`
       └─ IntervalEnd (closed)
          └─ Numeric
             └─ `48000`
    "#;
  assert_eq!(expected, node.trace());
}

#[test]
fn _0004() {
  let node = dsntk_feel_parser::parse_range_literal(&scope!(), "(48000..70000]", false).unwrap();
  let expected = r#"
       Range
       ├─ IntervalStart (opened)
       │  └─ Numeric
       │     └─ `48000`
       └─ IntervalEnd (closed)
          └─ Numeric
             └─ `70000`
    "#;
  assert_eq!(expected, node.trace());
}

#[test]
fn _0005() {
  let node = dsntk_feel_parser::parse_range_literal(&scope!(), "(70000..180000]", false).unwrap();
  let expected = r#"
       Range
       ├─ IntervalStart (opened)
       │  └─ Numeric
       │     └─ `70000`
       └─ IntervalEnd (closed)
          └─ Numeric
             └─ `180000`
    "#;
  assert_eq!(expected, node.trace());
}

#[test]
fn _0006() {
  let node = dsntk_feel_parser::parse_range_literal(&scope!(), "(180000..)", false).unwrap();
  let expected = r#"
       Range
       ├─ IntervalStart (opened)
       │  └─ Numeric
       │     └─ `180000`
       └─ IntervalEnd (opened)
          └─ Null
    "#;
  assert_eq!(expected, node.trace());
}

#[test]
fn _0007() {
  let node = dsntk_feel_parser::parse_range_literal(&scope!(), "[0..)", false).unwrap();
  let expected = r#"
       Range
       ├─ IntervalStart (closed)
       │  └─ Numeric
       │     └─ `0`
       └─ IntervalEnd (opened)
          └─ Null
    "#;
  assert_eq!(expected, node.trace());
}

#[test]
fn _0008() {
  let node = dsntk_feel_parser::parse_range_literal(&scope!(), "(..1000]", false).unwrap();
  let expected = r#"
       Range
       ├─ IntervalStart (opened)
       │  └─ Null
       └─ IntervalEnd (closed)
          └─ Numeric
             └─ `1000`
    "#;
  assert_eq!(expected, node.trace());
}

#[test]
fn _0009() {
  let node = dsntk_feel_parser::parse_range_literal(&scope!(), "(..)", false).unwrap();
  let expected = r#"
       Range
       ├─ IntervalStart (opened)
       │  └─ Null
       └─ IntervalEnd (opened)
          └─ Null
    "#;
  assert_eq!(expected, node.trace());
}

#[test]
fn _0010() {
  let node = dsntk_feel_parser::parse_range_literal(&scope!(), "[1000..2000)", false).unwrap();
  let expected = r#"
       Range
       ├─ IntervalStart (closed)
       │  └─ Numeric
       │     └─ `1000`
       └─ IntervalEnd (opened)
          └─ Numeric
             └─ `2000`
    "#;
  assert_eq!(expected, node.trace());
}

#[test]
fn _0011() {
  let node = dsntk_feel_parser::parse_unary_tests(&scope!(), "(180000..)", false).unwrap();
  let expected = r#"
       ExpressionList
       └─ Range
          ├─ IntervalStart (opened)
          │  └─ Numeric
          │     └─ `180000`
          └─ IntervalEnd (opened)
             └─ Null
    "#;
  assert_eq!(expected, node.trace());
}

#[test]
fn _0012() {
  let node = dsntk_feel_parser::parse_unary_tests(&scope!(), "(..1000]", false).unwrap();
  let expected = r#"
       ExpressionList
       └─ Range
          ├─ IntervalStart (opened)
          │  └─ Null
          └─ IntervalEnd (closed)
             └─ Numeric
                └─ `1000`
    "#;
  assert_eq!(expected, node.trace());
}

#[test]
fn _0013() {
  let node = dsntk_feel_parser::parse_unary_tests(&scope!(), "(..)", false).unwrap();
  let expected = r#"
       ExpressionList
       └─ Range
          ├─ IntervalStart (opened)
          │  └─ Null
          └─ IntervalEnd (opened)
             └─ Null
    "#;
  assert_eq!(expected, node.trace());
}
