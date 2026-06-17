// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Assembly pipeline trait and implementations.
//!
//! This module provides the `AssemblyPipeline` trait for abstracting
//! tile assembly. It enables testing assembly logic without real
//! JPEG decoding or DDS compression.

use crate::tiles::assembler::{AssemblyConfig, AssemblyError, AssemblyResult};
use async_trait::async_trait;
use thiserror::Error;

/// Errors that can occur during assembly operations.
#[derive(Debug, Error)]
pub enum AssemblyServiceError {
    #[error("Assembly error: {0}")]
    Assembly(#[from] AssemblyError),

    #[error("Pipeline error: {0}")]
    Pipeline(String),
}

/// Result type for assembly service operations.
pub type AssemblyServiceResult<T> = Result<T, AssemblyServiceError>;

/// Service trait for tile assembly.
///
/// This trait abstracts over the tile assembly pipeline, enabling
/// testing with mock implementations that return deterministic output.
#[async_trait]
pub trait AssemblyPipeline: Send + Sync {
    /// Assemble JPEG chunks into a complete DDS tile.
    ///
    /// - `chunks`: Row-major array of optional JPEG data (None = use fallback color)
    /// - `config`: Assembly configuration (chunk size, format, etc.)
    async fn assemble(
        &self,
        chunks: &[Option<Vec<u8>>],
        config: &AssemblyConfig,
    ) -> AssemblyServiceResult<AssemblyResult>;
}

/// Production implementation wrapping the free function `assemble_tile`.
pub struct AssemblyPipelineImpl;

impl AssemblyPipelineImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AssemblyPipelineImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AssemblyPipeline for AssemblyPipelineImpl {
    async fn assemble(
        &self,
        chunks: &[Option<Vec<u8>>],
        config: &AssemblyConfig,
    ) -> AssemblyServiceResult<AssemblyResult> {
        // Delegate to the free function (runs synchronously via rayon internally)
        crate::tiles::assembler::assemble_tile(chunks, config).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::dds::DdsFormat;

    /// Fake assembly pipeline for testing without real decoding/compression.
    pub struct FakeAssemblyPipeline {
        /// Predefined DDS data to return (if set)
        dds_data: Option<Vec<u8>>,
        /// Number of times assemble was called
        call_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl FakeAssemblyPipeline {
        pub fn new() -> Self {
            Self {
                dds_data: None,
                call_count: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            }
        }

        /// Create with specific DDS data to return.
        pub fn with_dds_data(data: Vec<u8>) -> Self {
            Self {
                dds_data: Some(data),
                call_count: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            }
        }

        /// Get the number of times assemble was called.
        pub fn call_count(&self) -> usize {
            self.call_count.load(std::sync::atomic::Ordering::Relaxed)
        }

        /// Create a default result for testing.
        pub fn default_result() -> AssemblyResult {
            AssemblyResult {
                dds_data: vec![0u8; 148], // Minimal DDS header
                chunks_decoded: 256,
                chunks_failed: 0,
                mipmap_count: 13,
            }
        }
    }

    impl Default for FakeAssemblyPipeline {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl AssemblyPipeline for FakeAssemblyPipeline {
        async fn assemble(
            &self,
            _chunks: &[Option<Vec<u8>>],
            _config: &AssemblyConfig,
        ) -> AssemblyServiceResult<AssemblyResult> {
            self.call_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            match &self.dds_data {
                Some(data) => Ok(AssemblyResult {
                    dds_data: data.clone(),
                    chunks_decoded: 256,
                    chunks_failed: 0,
                    mipmap_count: 13,
                }),
                None => Ok(FakeAssemblyPipeline::default_result()),
            }
        }
    }

    #[tokio::test]
    async fn test_fake_pipeline_default_result() {
        let pipeline = FakeAssemblyPipeline::new();
        let config = AssemblyConfig::default();

        let result = pipeline.assemble(&vec![None; 256], &config).await.unwrap();

        assert_eq!(result.chunks_decoded, 256);
        assert_eq!(result.chunks_failed, 0);
        assert_eq!(result.mipmap_count, 13);
        assert_eq!(pipeline.call_count(), 1);
    }

    #[tokio::test]
    async fn test_fake_pipeline_custom_data() {
        let pipeline = FakeAssemblyPipeline::with_dds_data(vec![0x41, 0x42, 0x43]);
        let config = AssemblyConfig::default();

        let result = pipeline.assemble(&vec![None; 256], &config).await.unwrap();

        assert_eq!(result.dds_data, vec![0x41, 0x42, 0x43]);
    }

    #[tokio::test]
    async fn test_production_pipeline() {
        let pipeline = AssemblyPipelineImpl::new();
        let config = AssemblyConfig {
            chunks_per_side: 16,
            chunk_size: 256,
            format: DdsFormat::BC1,
            missing_color: [66, 77, 55],
            seasonal_saturation: 1.0,
        };

        // All missing chunks → fallback color
        let result = pipeline.assemble(&vec![None; 256], &config).await.unwrap();

        assert_eq!(result.chunks_decoded, 0);
        assert_eq!(result.chunks_failed, 256);
        assert!(!result.dds_data.is_empty());
    }

    /// Test that AssemblyPipeline works as a trait object.
    #[tokio::test]
    async fn test_assembly_pipeline_trait_object() {
        let fake = FakeAssemblyPipeline::new();
        let pipeline: Box<dyn AssemblyPipeline> = Box::new(fake);
        let config = AssemblyConfig::default();

        let result = pipeline.assemble(&vec![None; 256], &config).await.unwrap();

        assert_eq!(result.chunks_decoded, 256);
    }

    /// Test that we can swap implementations without changing client code.
    #[tokio::test]
    async fn test_assembly_pipeline_impl_swap() {
        async fn run_pipeline<P: AssemblyPipeline>(pipeline: &P) -> u32 {
            let config = AssemblyConfig::default();
            let result = pipeline.assemble(&vec![None; 256], &config).await.unwrap();
            result.chunks_decoded
        }

        let fake = FakeAssemblyPipeline::new();
        assert_eq!(run_pipeline(&fake).await, 256);

        let real = AssemblyPipelineImpl::new();
        assert_eq!(run_pipeline(&real).await, 0); // All missing → 0 decoded
    }
}
