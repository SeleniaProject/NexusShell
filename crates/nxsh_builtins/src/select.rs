use anyhow::Result;
use jmespath::{Variable, Expression};
use serde_json::Value;
use std::io::{self, Read};

pub fn select_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        anyhow::bail!("select requires JMESPath expression");
    }
    let expr_str = &args[0];
    let expr = jmespath::compile(expr_str)?;

    // Read all stdin
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let json: Value = serde_json::from_str(&buf)?;
    let data = Variable::from(json);
    let result = expr.search(data)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
} 