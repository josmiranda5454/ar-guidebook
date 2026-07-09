use crate::models::{Area, OfflinePack};
use async_trait::async_trait;
use std::{error::Error, fmt};
use uuid::Uuid;

pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[derive(Debug)]
pub enum RepositoryError {
    Database(sqlx::Error),
    Decode(String),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(f, "database error: {error}"),
            Self::Decode(error) => write!(f, "decode error: {error}"),
        }
    }
}

impl Error for RepositoryError {}

impl From<sqlx::Error> for RepositoryError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

#[async_trait]
pub trait GuideRepository: Send + Sync {
    async fn areas(&self) -> RepositoryResult<Vec<Area>>;
    async fn area(&self, area_id: Uuid) -> RepositoryResult<Option<Area>>;
    async fn offline_pack(&self, area_id: Uuid) -> RepositoryResult<Option<OfflinePack>>;
}
