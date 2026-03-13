use crate::schema::CommandSchema;
use serde::Serialize;

/// Description of an operation for dry-run output.
#[derive(Debug, Clone, Serialize)]
pub struct OperationDescription {
    pub operation: String,
    pub params: serde_json::Value,
    pub description: String,
}

/// Trait for processing operations, generic over data type and error type.
pub trait Operation<T, E>: Send + Sync {
    /// Human-readable name.
    fn name(&self) -> &str;

    /// Apply the operation to the input data.
    fn apply(&self, input: T) -> Result<T, E>;

    /// Describe what this operation will do (for dry-run).
    fn describe(&self) -> OperationDescription;

    /// Parameter schema for this operation.
    fn schema() -> CommandSchema
    where
        Self: Sized;
}

/// Dry-run plan output.
#[derive(Debug, Serialize)]
pub struct PipelinePlan {
    pub steps: Vec<OperationDescription>,
}

/// A pipeline of operations to be applied in order.
pub struct Pipeline<T, E> {
    operations: Vec<Box<dyn Operation<T, E>>>,
}

impl<T, E> Pipeline<T, E> {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn push<O: Operation<T, E> + 'static>(mut self, op: O) -> Self {
        self.operations.push(Box::new(op));
        self
    }

    pub fn push_boxed(mut self, op: Box<dyn Operation<T, E>>) -> Self {
        self.operations.push(op);
        self
    }

    /// Execute all operations in order on the given input.
    pub fn execute(&self, mut input: T) -> Result<T, E> {
        for op in &self.operations {
            input = op.apply(input)?;
        }
        Ok(input)
    }

    /// Return a plan of what would be executed (for --dry-run).
    pub fn describe(&self) -> PipelinePlan {
        PipelinePlan {
            steps: self.operations.iter().map(|op| op.describe()).collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.operations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

impl<T, E> Default for Pipeline<T, E> {
    fn default() -> Self {
        Self::new()
    }
}
