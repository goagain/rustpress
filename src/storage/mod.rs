//! Storage abstraction layer for file uploads
//! 
//! This module provides a trait-based storage interface that allows
//! easy switching between different storage backends (local filesystem, S3, etc.)

pub mod local_storage;
pub mod storage_trait;

pub use storage_trait::*;
pub use local_storage::*;

