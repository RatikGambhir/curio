//! Space use cases. The caller supplies the authenticated owner separately from input.
use super::model::{
    CreateSpaceInput, ListSpacesQuery, NewSpace, Space, SpaceCursor, SpaceError, SpacePage,
    SpacesResponse,
};

pub trait SpaceStore: Send + Sync {
    fn insert(
        &self,
        owner: &str,
        space: NewSpace<'_>,
    ) -> impl Future<Output = Result<Space, SpaceError>> + Send;
    fn list(
        &self,
        owner: &str,
        page: &SpacePage,
    ) -> impl Future<Output = Result<Vec<Space>, SpaceError>> + Send;
    fn delete(&self, owner: &str, id: &str) -> impl Future<Output = Result<(), SpaceError>> + Send;
}

pub struct SpaceService<R> {
    repository: R,
}

impl<R: SpaceStore> SpaceService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn create(&self, owner: &str, input: CreateSpaceInput) -> Result<Space, SpaceError> {
        self.repository
            .insert(owner, NewSpace::parse(&input)?)
            .await
    }

    pub async fn list(
        &self,
        owner: &str,
        query: ListSpacesQuery,
    ) -> Result<SpacesResponse, SpaceError> {
        let page = SpacePage::parse(query)?;
        let mut spaces = self.repository.list(owner, &page).await?;
        let next_cursor = if spaces.len() > page.limit() {
            spaces.truncate(page.limit());
            spaces.last().map(SpaceCursor::encode).transpose()?
        } else {
            None
        };
        Ok(SpacesResponse {
            spaces,
            next_cursor,
        })
    }

    pub async fn delete(&self, owner: &str, id: &str) -> Result<(), SpaceError> {
        self.repository.delete(owner, id).await
    }
}
