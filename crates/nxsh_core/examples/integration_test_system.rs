fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "heavy-time"))]
    {
        println!("(integration_test_system example built without heavy-time feature)");
    }
    #[cfg(feature = "heavy-time")]
    println!("Minimal integration test system demo (heavy-time enabled)");
    Ok(())
}
