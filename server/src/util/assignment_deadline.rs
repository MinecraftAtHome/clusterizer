use crate::state::AppState;

pub async fn update_expired_assignments(state: &AppState) -> sqlx::Result<()> {
    sqlx::query_unchecked!(
        r#"
        UPDATE
            assignments
        SET
            state = 'expired'
        WHERE
            state = 'init'
            AND deadline_at < now()
        "#
    )
    .execute(&state.pool)
    .await?;

    Ok(())
}
