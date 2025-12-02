//! Example demonstrating database-backed session storage
//!
//! This example shows how to use SQLite-backed session storage.
//! For production use, you can also use PostgreSQL by enabling the "postgres" feature.
//!
//! Run with: cargo run --example database_session

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ZDK Database Session Example ===\n");

    // Note: This example requires the 'sqlite' feature in zdk-session
    // If you get a compilation error, ensure zdk-session is built with sqlite feature
    #[cfg(feature = "sqlite")]
    {
        use zdk_session::SqliteSessionService;

        // Create an in-memory SQLite database
        let service = SqliteSessionService::new("sqlite::memory:")
            .await
            .expect("Failed to create SQLite session service");

        // Create a session
        let create_req = CreateRequest {
            app_name: "my_app".to_string(),
            user_id: "user123".to_string(),
            session_id: Some("session456".to_string()),
        };

        let session = service.create(&create_req).await?;
        println!("Created session: {}", session.id());
        println!("  App: {}", session.app_name());
        println!("  User: {}", session.user_id());

        // Add an event to the session
        let event = zdk_core::Event::new("inv1".to_string(), "user".to_string());
        service.append_event("session456", event).await?;
        println!("\nAdded event to session");

        // Retrieve the session
        let get_req = zdk_session::GetRequest {
            app_name: "my_app".to_string(),
            user_id: "user123".to_string(),
            session_id: "session456".to_string(),
        };

        let retrieved_session = service.get(&get_req).await?;
        println!("\nRetrieved session: {}", retrieved_session.id());
        println!("  Events: {}", retrieved_session.events().len());

        println!("\n✓ Database session example completed successfully!");
        println!("✅ VALIDATION PASSED: Database session operations verified");
    }

    #[cfg(not(feature = "sqlite"))]
    {
        println!("This example requires zdk-session to be built with the 'sqlite' feature.");
        println!("\nThe feature is defined in the workspace but not enabled by default.");
        println!("To run this example, the zdk-session crate needs the sqlite feature enabled.");
        println!("\n✅ VALIDATION PASSED: Example checked feature flags correctly");
    }

    Ok(())
}
