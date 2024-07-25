use std::path::Path;

use once_cell::sync::Lazy;
use oxc::{
  ast::ast::{Argument, BinaryExpression, CallExpression, Expression, TemplateLiteral},
  syntax::operator::BinaryOperator,
};
use regex::Regex;
use anyhow;

use crate::sanitize::sanitize_string;
use crate::should_ignore::should_ignore;

// Disallow ./*.ext
static OWN_DIRECTORY_STAR_REGEX: Lazy<Regex> = Lazy::new(|| {
  let pattern: &str = "^\\.\\/\\*\\.[\\w]+$";
  Regex::new(pattern).expect("failed to compile regex")
});

static EXAMPLE_CODE: &str = "For example: import(`./foo/${bar}.js`).";

pub fn to_glob_pattern<'ast>(expr: &Expression<'ast>) -> anyhow::Result<Option<String>> {
  let glob = expr_to_glob(expr)?;

  if should_ignore(&glob) {
    return Ok(None);
  }

  let glob = glob.replace("**", "*");

  if glob.starts_with('*') {
    return Err(
      anyhow::format_err!("invalid import \"{expr:?}\". It cannot be statically analyzed. Variable dynamic imports must start with ./ and be limited to a specific directory. {EXAMPLE_CODE}"));
  }

  if glob.ends_with('/') {
    return Err(
      anyhow::format_err!("invalid import \"{expr:?}\". Variable absolute imports are not supported, imports must start with ./ in the static part of the import. {EXAMPLE_CODE}"));
  }

  if !glob.starts_with("./") && !glob.starts_with("../") {
    return Err(
      anyhow::format_err!("invalid import \"{expr:?}\". Variable bare imports are not supported, imports must start with ./ in the static part of the import. {EXAMPLE_CODE}"));
  }

  if OWN_DIRECTORY_STAR_REGEX.is_match(&glob) {
    return Err(
      anyhow::format_err!("invalid import \"{expr:?}\". Variable imports cannot import their own directory, place imports in a separate directory or make the import filename more specific. {EXAMPLE_CODE}"));
  }

  if Path::new(&glob).extension().is_none() {
    return Err(
      anyhow::format_err!("invalid import \"{expr:?}\". A file extension must be included in the static part of the import. {EXAMPLE_CODE}"),
    );
  }

  Ok(Some(glob))
}

fn expr_to_glob<'ast>(expr: &Expression<'ast>) -> anyhow::Result<String> {
  Ok(match expr {
    Expression::TemplateLiteral(node) => template_literal_to_glob(node)?,
    Expression::CallExpression(node) => call_expr_to_glob(node)?,
    Expression::BinaryExpression(node) => binary_expr_to_glob(node)?,
    Expression::StringLiteral(node) => sanitize_string(&node.value)?,
    _ => String::from("*"),
  })
}

fn arg_to_glob<'ast>(arg: &Argument<'ast>) -> anyhow::Result<String> {
  Ok(match arg {
    Argument::TemplateLiteral(node) => template_literal_to_glob(node)?,
    Argument::CallExpression(node) => call_expr_to_glob(node)?,
    Argument::BinaryExpression(node) => binary_expr_to_glob(node)?,
    Argument::StringLiteral(node) => sanitize_string(&node.value)?,
    _ => String::from("*"),
  })
}

fn template_literal_to_glob<'ast>(node: &TemplateLiteral<'ast>) -> anyhow::Result<String> {
  let mut glob = String::new();

  for (index, quasi) in node.quasis.iter().enumerate() {
    glob += &sanitize_string(&quasi.value.raw)?;
    if let Some(expr) = node.expressions.get(index) {
      glob += &expr_to_glob(expr)?;
    }
  }

  Ok(glob)
}

fn call_expr_to_glob<'ast>(node: &CallExpression<'ast>) -> anyhow::Result<String> {
  if let Expression::StaticMemberExpression(member_expr) = &node.callee {
    if member_expr.property.name == "concat" {
      let mut arg_glob = String::new();
      for arg in &node.arguments {
        arg_glob += &arg_to_glob(arg)?;
      }
      return Ok(expr_to_glob(&member_expr.object)? + &arg_glob);
    }
  }

  Ok(String::from("*"))
}

fn binary_expr_to_glob<'ast>(node: &BinaryExpression<'ast>) -> anyhow::Result<String> {
  if !matches!(node.operator, BinaryOperator::Addition) {
    return Err(anyhow::format_err!("{:?} operator is not supported.", node.operator));
  }

  Ok(expr_to_glob(&node.left)? + &expr_to_glob(&node.right)?)
}