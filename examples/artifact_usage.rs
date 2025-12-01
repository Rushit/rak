//! Example demonstrating artifact storage usage

use zdk_artifact::{
    ArtifactPart, ArtifactService, FileSystemArtifactService, InMemoryArtifactService, ListRequest,
    LoadRequest, SaveRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ZDK Artifact Service Example ===\n");

    // Example 1: In-memory artifact service
    println!("1. Using InMemoryArtifactService:");
    let memory_service = InMemoryArtifactService::new();
    demonstrate_artifact_service(&memory_service).await?;

    println!("\n2. Using FileSystemArtifactService:");
    let temp_dir = tempfile::tempdir()?;
    let fs_service = FileSystemArtifactService::new(temp_dir.path());
    demonstrate_artifact_service(&fs_service).await?;

    Ok(())
}

async fn demonstrate_artifact_service(
    service: &impl ArtifactService,
) -> Result<(), Box<dyn std::error::Error>> {
    // Save a text artifact
    let save_req = SaveRequest {
        app_name: "my_app".to_string(),
        user_id: "user123".to_string(),
        session_id: "session456".to_string(),
        file_name: "document.txt".to_string(),
        part: ArtifactPart::text("Hello, ZDK!"),
        version: None,
    };

    let save_resp = service.save(save_req).await?;
    println!("  Saved artifact, version: {}", save_resp.version);

    // Save another version
    let save_req = SaveRequest {
        app_name: "my_app".to_string(),
        user_id: "user123".to_string(),
        session_id: "session456".to_string(),
        file_name: "document.txt".to_string(),
        part: ArtifactPart::text("Hello, ZDK v2!"),
        version: None,
    };

    let save_resp = service.save(save_req).await?;
    println!("  Saved artifact, version: {}", save_resp.version);

    // Load the latest version
    let load_req = LoadRequest {
        app_name: "my_app".to_string(),
        user_id: "user123".to_string(),
        session_id: "session456".to_string(),
        file_name: "document.txt".to_string(),
        version: None,
    };

    let load_resp = service.load(load_req).await?;
    println!("  Loaded artifact: {:?}", load_resp.part);

    // Save a binary artifact
    let image_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header
    let save_req = SaveRequest {
        app_name: "my_app".to_string(),
        user_id: "user123".to_string(),
        session_id: "session456".to_string(),
        file_name: "image.jpg".to_string(),
        part: ArtifactPart::binary("image/jpeg", image_data),
        version: None,
    };

    service.save(save_req).await?;
    println!("  Saved binary artifact");

    // List all artifacts in the session
    let list_req = ListRequest {
        app_name: "my_app".to_string(),
        user_id: "user123".to_string(),
        session_id: "session456".to_string(),
    };

    let list_resp = service.list(list_req).await?;
    println!("  Artifacts in session: {:?}", list_resp.file_names);

    // User-scoped artifact
    let save_req = SaveRequest {
        app_name: "my_app".to_string(),
        user_id: "user123".to_string(),
        session_id: "session456".to_string(),
        file_name: "user:profile.json".to_string(),
        part: ArtifactPart::text(r#"{"name": "John Doe"}"#),
        version: None,
    };

    service.save(save_req).await?;
    println!("  Saved user-scoped artifact");

    Ok(())
}
