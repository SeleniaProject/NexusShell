// Shell module placeholder for build compatibility
use crate::compat::Result;

pub struct Shell;

impl Shell {
    pub fn new() -> Self {
        Shell
    }
    
    pub async fn run(&self) -> Result<()> {
        Ok(())
    }
}
