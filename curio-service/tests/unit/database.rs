use crate::postgres_test_support::PostgresFixture;

#[tokio::test]
async fn readiness_requires_the_latest_migration() {
    let Some(postgres) =
        PostgresFixture::provision("readiness_requires_the_latest_migration").await
    else {
        return;
    };
    let expected_version = crate::database::MIGRATOR
        .iter()
        .last()
        .expect("PostgreSQL migrations must exist")
        .version;
    assert_eq!(expected_version, 202608310002);

    sqlx::query("DELETE FROM _sqlx_migrations WHERE version = $1")
        .bind(expected_version)
        .execute(postgres.database().pool())
        .await
        .unwrap();

    assert!(postgres.database().readiness().await.is_err());
    postgres.cleanup().await;
}
