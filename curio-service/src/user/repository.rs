use crate::{
    core::sql::Sql,
    database::Database,
    user::model::{SaveUser, UserRecord},
};

#[derive(Clone)]
pub struct UserRepository {
    database: Database,
}

impl UserRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn save(&self, user: SaveUser<'_>) -> Result<UserRecord, sqlx::Error> {
        Sql::insert_into("users")
            .set("id", user.id)
            .set("name", user.name)
            .set("email", user.email)
            .set("avatar_url", user.avatar_url)
            .upsert_on("id")
            .returning(UserRecord::COLUMNS)
            .fetch_one(self.database.pool())
            .await
    }
}
