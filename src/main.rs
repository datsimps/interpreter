use anyhow::Result;

fn main() -> Result<()> {
    let _ = interpreter::token::repl::start();
    
    Ok(())
}
