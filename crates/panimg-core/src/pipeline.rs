use crate::error::Result;
use crate::ops::{Operation, OperationDescription};
use image::DynamicImage;
use serde::Serialize;

/// A pipeline of image operations to be applied in order.
pub struct Pipeline {
    operations: Vec<Box<dyn Operation>>,
}

/// Dry-run plan output.
#[derive(Debug, Serialize)]
pub struct PipelinePlan {
    pub steps: Vec<OperationDescription>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn push<O: Operation + 'static>(mut self, op: O) -> Self {
        self.operations.push(Box::new(op));
        self
    }

    pub fn push_boxed(mut self, op: Box<dyn Operation>) -> Self {
        self.operations.push(op);
        self
    }

    /// Execute all operations in order on the given image.
    pub fn execute(&self, mut img: DynamicImage) -> Result<DynamicImage> {
        for op in &self.operations {
            img = op.apply(img)?;
        }
        Ok(img)
    }

    /// Return a plan of what would be executed (for --dry-run).
    pub fn describe(&self) -> PipelinePlan {
        PipelinePlan {
            steps: self.operations.iter().map(|op| op.describe()).collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}
