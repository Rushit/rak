//! Capability type definitions for provider system
//!
//! Defines types for different provider capabilities: embeddings, transcription,
//! image generation, and audio generation.

use serde::{Deserialize, Serialize};

/// Embedding vector result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingVector {
    /// The embedding vector values
    pub vector: Vec<f32>,
    /// Number of dimensions in the vector
    pub dimensions: usize,
}

impl EmbeddingVector {
    /// Create a new embedding vector
    pub fn new(vector: Vec<f32>) -> Self {
        let dimensions = vector.len();
        Self { vector, dimensions }
    }
}

/// Audio input for transcription
#[derive(Debug, Clone)]
pub struct AudioInput {
    /// Audio data bytes
    pub data: Vec<u8>,
    /// Audio format (e.g., "mp3", "wav", "m4a", "webm")
    pub format: String,
    /// Optional language hint for transcription
    pub language: Option<String>,
}

/// Transcription result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    /// Transcribed text
    pub text: String,
    /// Detected or specified language
    pub language: Option<String>,
    /// Audio duration in seconds
    pub duration: Option<f32>,
    /// Optional segments with timestamps
    pub segments: Option<Vec<TranscriptionSegment>>,
}

/// Transcription segment with timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    /// Segment text
    pub text: String,
    /// Start time in seconds
    pub start: f32,
    /// End time in seconds
    pub end: f32,
}

/// Image generation request
#[derive(Debug, Clone)]
pub struct ImageRequest {
    /// Text prompt for image generation
    pub prompt: String,
    /// Image size (e.g., "1024x1024", "512x512")
    pub size: Option<String>,
    /// Quality setting (e.g., "standard", "hd")
    pub quality: Option<String>,
    /// Style setting (e.g., "natural", "vivid")
    pub style: Option<String>,
    /// Number of images to generate
    pub n: Option<u32>,
}

/// Image generation result
#[derive(Debug, Clone)]
pub struct ImageResult {
    /// Generated images
    pub images: Vec<GeneratedImage>,
}

/// A single generated image
#[derive(Debug, Clone)]
pub struct GeneratedImage {
    /// Image data as PNG bytes
    pub data: Vec<u8>,
    /// Optional URL if image is hosted
    pub url: Option<String>,
    /// Revised prompt used for generation
    pub revised_prompt: Option<String>,
}

/// Audio generation request
#[derive(Debug, Clone)]
pub struct AudioRequest {
    /// Text to convert to speech
    pub text: String,
    /// Voice to use (provider-specific)
    pub voice: Option<String>,
    /// Speed of speech (0.25 to 4.0, 1.0 is normal)
    pub speed: Option<f32>,
}

/// Audio generation result
#[derive(Debug, Clone)]
pub struct AudioResult {
    /// Audio data bytes
    pub data: Vec<u8>,
    /// Audio format (e.g., "mp3", "opus", "aac", "flac")
    pub format: String,
}
