use std::time::Duration;

use tokio::time;

use crate::state::AppState;

pub async fn update_expired_assignments(state: AppState) {
    let mut interval = time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

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
        .await
        .unwrap();
    }
}
