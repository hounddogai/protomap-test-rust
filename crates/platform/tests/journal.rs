//! Exercises the migration and the repository query against an isolated, real PostgreSQL database.

#[tokio::test]
async fn test_journal_preserves_tenant_and_payload() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("ACME_TEST_DATABASE_URL")?;
    let pool = sqlx::PgPool::connect(&url).await?;
    let journal = acme_platform::journal::Journal::new(pool.clone());
    journal.migrate().await?;
    journal.migrate().await?;
    journal.append("secrets", "rotate", "tenant-a", vec![0, 255, 1]).await?;
    journal.append("secrets", "rotate", "tenant-b", vec![42]).await?;
    let rows = sqlx::query_as::<_, (String, String, String, Vec<u8>)>(
        "SELECT domain, method, tenant_id, payload FROM acme_audit ORDER BY id",
    )
    .fetch_all(&pool)
    .await?;
    assert_eq!(
        rows,
        vec![
            ("secrets".into(), "rotate".into(), "tenant-a".into(), vec![0, 255, 1]),
            ("secrets".into(), "rotate".into(), "tenant-b".into(), vec![42]),
        ]
    );
    pool.close().await;
    Ok(())
}
