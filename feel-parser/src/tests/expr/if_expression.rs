use super::super::*;

#[test]
fn _0001() {
  let scope = scope!();
  accept(
    &scope,
    StartExpression,
    "if 1 > 2 then 5 else 8",
    r#"
       If
       ├─ Gt
       │  ├─ Numeric
       │  │  └─ `1`
       │  └─ Numeric
       │     └─ `2`
       ├─ Numeric
       │  └─ `5`
       └─ Numeric
          └─ `8`
    "#,
    false,
  );
}

#[test]
fn _0002() {
  let scope = scope!();
  scope.set_entry_name("a".into());
  scope.set_entry_name("b".into());
  accept(
    &scope,
    StartExpression,
    "if a > b then a else b",
    r#"
       If
       ├─ Gt
       │  ├─ Name
       │  │  └─ `a`
       │  └─ Name
       │     └─ `b`
       ├─ Name
       │  └─ `a`
       └─ Name
          └─ `b`
    "#,
    false,
  );
}

#[test]
fn _0003() {
  // Conditional expression with a bare name as the condition.
  let scope = scope!();
  scope.set_entry_name("eligible".into());
  accept(
    &scope,
    StartExpression,
    "if eligible then 1 else 2",
    r#"
       If
       ├─ Name
       │  └─ `eligible`
       ├─ Numeric
       │  └─ `1`
       └─ Numeric
          └─ `2`
    "#,
    false,
  );
}

#[test]
fn _0004() {
  // Conditional expression with a conjunction of names as the condition.
  let scope = scope!();
  scope.set_entry_name("a".into());
  scope.set_entry_name("b".into());
  accept(
    &scope,
    StartExpression,
    "if a and b then 1 else 2",
    r#"
       If
       ├─ And
       │  ├─ Name
       │  │  └─ `a`
       │  └─ Name
       │     └─ `b`
       ├─ Numeric
       │  └─ `1`
       └─ Numeric
          └─ `2`
    "#,
    false,
  );
}

#[test]
fn _0005() {
  // Conditional expression with a negated name as the condition.
  let scope = scope!();
  scope.set_entry_name("a".into());
  accept(
    &scope,
    StartExpression,
    "if not a then 1 else 2",
    r#"
       If
       ├─ FunctionInvocation
       │  ├─ Name
       │  │  └─ `not`
       │  └─ PositionalParameters
       │     └─ Name
       │        └─ `a`
       ├─ Numeric
       │  └─ `1`
       └─ Numeric
          └─ `2`
    "#,
    false,
  );
}

#[test]
fn _0006() {
  // Conditional expression with a bare-name condition and literal string branches.
  let scope = scope!();
  scope.set_entry_name("eligible".into());
  accept(
    &scope,
    StartExpression,
    r#"if eligible then "x" else "y""#,
    r#"
       If
       ├─ Name
       │  └─ `eligible`
       ├─ String
       │  └─ `x`
       └─ String
          └─ `y`
    "#,
    false,
  );
}

#[test]
fn _0007() {
  // Nested conditional expression in the then-branch.
  let scope = scope!();
  scope.set_entry_name("a".into());
  scope.set_entry_name("b".into());
  accept(
    &scope,
    StartExpression,
    "if a > 1 then (if b > 2 then 1 else 2) else 3",
    r#"
       If
       ├─ Gt
       │  ├─ Name
       │  │  └─ `a`
       │  └─ Numeric
       │     └─ `1`
       ├─ If
       │  ├─ Gt
       │  │  ├─ Name
       │  │  │  └─ `b`
       │  │  └─ Numeric
       │  │     └─ `2`
       │  ├─ Numeric
       │  │  └─ `1`
       │  └─ Numeric
       │     └─ `2`
       └─ Numeric
          └─ `3`
    "#,
    false,
  );
}

#[test]
fn _0008() {
  // Broken conditional expressions must still be rejected.
  let scope = scope!();
  scope.set_entry_name("a".into());
  for input in ["if then else", "if a then 1", "if a then else 2", "if then 1 else 2"] {
    assert!(
      Parser::new(&scope, StartExpression, input, false).parse().is_err(),
      "expected syntax error for input: {input}"
    );
  }
}

#[test]
fn _0009() {
  // Conditional expression with a disjunction as the condition.
  let scope = scope!();
  scope.set_entry_name("a".into());
  scope.set_entry_name("b".into());
  accept(
    &scope,
    StartExpression,
    "if a or b then 1 else 2",
    r#"
       If
       ├─ Or
       │  ├─ Name
       │  │  └─ `a`
       │  └─ Name
       │     └─ `b`
       ├─ Numeric
       │  └─ `1`
       └─ Numeric
          └─ `2`
    "#,
    false,
  );
}

#[test]
fn _0010() {
  // Nested conditional expression as the condition, without parentheses.
  let scope = scope!();
  scope.set_entry_name("a".into());
  scope.set_entry_name("b".into());
  accept(
    &scope,
    StartExpression,
    "if if b > 2 then true else false then 1 else 2",
    r#"
       If
       ├─ If
       │  ├─ Gt
       │  │  ├─ Name
       │  │  │  └─ `b`
       │  │  └─ Numeric
       │  │     └─ `2`
       │  ├─ Boolean
       │  │  └─ `true`
       │  └─ Boolean
       │     └─ `false`
       ├─ Numeric
       │  └─ `1`
       └─ Numeric
          └─ `2`
    "#,
    false,
  );
}

#[test]
fn _9999_default_scope_repro() {
  // The dsntk-rs#14 repro: the same expressions parsed with a DEFAULT scope
  // (no registered entry names) — exactly how rexlang's rule-12 gate calls
  // `parse_expression`.
  for input in [
    "if eligible then 1 else 2",
    "if eligible and active then 1 else 2",
    "if not eligible then 1 else 2",
    r#"if eligible then "x" else "y""#,
    "if a > 1 then (if b > 2 then 1 else 2) else 3",
  ] {
    let scope = ParsingScope::default();
    let parsed = Parser::new(&scope, StartExpression, input, false).parse();
    assert!(parsed.is_ok(), "expected `{input}` to parse with a default scope: {parsed:?}");
  }
}
