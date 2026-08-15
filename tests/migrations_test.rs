use diesel::dsl::sql;
use diesel::prelude::*;
use noshi_back::shared::db::{establish_pool, run_migrations, DbPool};
use testcontainers::clients::Cli;
use testcontainers::Container;
use testcontainers_modules::postgres::Postgres;
use uuid::Uuid;

fn setup(docker: &Cli) -> (Container<'_, Postgres>, DbPool) {
    let container = docker.run(Postgres::default());
    let port = container.get_host_port_ipv4(5432);
    let db_url = format!("postgres://postgres:postgres@localhost:{}/postgres", port);
    let pool = establish_pool(&db_url, 4);
    run_migrations(&pool);
    (container, pool)
}

#[test]
fn migrations_run_successfully() {
    let docker = Cli::default();
    let (_container, pool) = setup(&docker);
    let mut conn = pool.get().expect("Failed to get connection");

    let count: i64 = sql::<diesel::sql_types::BigInt>(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_name IN ('users', 'sessions', 'password_resets', 'profiles', 'manual_platforms', 'waitlist')",
    )
    .get_result(&mut conn)
    .expect("Failed to count tables");

    assert_eq!(count, 6, "All 6 tables should exist");
}

#[test]
fn users_table_has_required_columns() {
    let docker = Cli::default();
    let (_container, pool) = setup(&docker);
    let mut conn = pool.get().expect("Failed to get connection");

    let count: i64 = sql::<diesel::sql_types::BigInt>(
        "SELECT COUNT(*) FROM information_schema.columns WHERE table_name = 'users' AND column_name IN ('id', 'email', 'name', 'password_hash', 'created_at', 'updated_at')",
    )
    .get_result(&mut conn)
    .expect("Failed to count columns");

    assert_eq!(count, 6, "users table should have 6 required columns");
}

#[test]
fn sessions_table_has_required_columns() {
    let docker = Cli::default();
    let (_container, pool) = setup(&docker);
    let mut conn = pool.get().expect("Failed to get connection");

    let count: i64 = sql::<diesel::sql_types::BigInt>(
        "SELECT COUNT(*) FROM information_schema.columns WHERE table_name = 'sessions' AND column_name IN ('id', 'user_id', 'refresh_token', 'expires_at', 'created_at')",
    )
    .get_result(&mut conn)
    .expect("Failed to count columns");

    assert_eq!(count, 5, "sessions table should have 5 required columns");
}

#[test]
fn password_resets_table_has_required_columns() {
    let docker = Cli::default();
    let (_container, pool) = setup(&docker);
    let mut conn = pool.get().expect("Failed to get connection");

    let count: i64 = sql::<diesel::sql_types::BigInt>(
        "SELECT COUNT(*) FROM information_schema.columns WHERE table_name = 'password_resets' AND column_name IN ('id', 'user_id', 'token_hash', 'expires_at', 'used_at', 'created_at')",
    )
    .get_result(&mut conn)
    .expect("Failed to count columns");

    assert_eq!(
        count, 6,
        "password_resets table should have 6 required columns"
    );
}

#[test]
fn profiles_table_has_required_columns() {
    let docker = Cli::default();
    let (_container, pool) = setup(&docker);
    let mut conn = pool.get().expect("Failed to get connection");

    let count: i64 = sql::<diesel::sql_types::BigInt>(
        "SELECT COUNT(*) FROM information_schema.columns WHERE table_name = 'profiles' AND column_name IN ('id', 'user_id', 'slug', 'niches', 'headline', 'is_published', 'completion_score', 'created_at', 'updated_at')",
    )
    .get_result(&mut conn)
    .expect("Failed to count columns");

    assert_eq!(count, 9, "profiles table should have 9 required columns");
}

#[test]
fn manual_platforms_table_has_required_columns() {
    let docker = Cli::default();
    let (_container, pool) = setup(&docker);
    let mut conn = pool.get().expect("Failed to get connection");

    let count: i64 = sql::<diesel::sql_types::BigInt>(
        "SELECT COUNT(*) FROM information_schema.columns WHERE table_name = 'manual_platforms' AND column_name IN ('id', 'user_id', 'platform', 'handle', 'follower_count', 'created_at', 'updated_at')",
    )
    .get_result(&mut conn)
    .expect("Failed to count columns");

    assert_eq!(
        count, 7,
        "manual_platforms table should have 7 required columns"
    );
}

#[test]
fn foreign_key_constraint_enforced() {
    let docker = Cli::default();
    let (_container, pool) = setup(&docker);
    let mut conn = pool.get().expect("Failed to get connection");

    let fake_user_id = Uuid::now_v7();
    let result = diesel::sql_query(
        "INSERT INTO sessions (id, user_id, refresh_token, expires_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::now_v7())
    .bind::<diesel::sql_types::Uuid, _>(fake_user_id)
    .bind::<diesel::sql_types::VarChar, _>("test-token")
    .execute(&mut conn);

    assert!(result.is_err(), "Should fail with foreign key violation");
}

#[test]
fn on_delete_cascade_works() {
    let docker = Cli::default();
    let (_container, pool) = setup(&docker);
    let mut conn = pool.get().expect("Failed to get connection");

    let user_id = Uuid::now_v7();
    let session_id = Uuid::now_v7();

    diesel::sql_query("INSERT INTO users (id, email, name, password_hash) VALUES ($1, $2, $3, $4)")
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::VarChar, _>("test@example.com")
        .bind::<diesel::sql_types::VarChar, _>("Test User")
        .bind::<diesel::sql_types::VarChar, _>("hashed_password")
        .execute(&mut conn)
        .expect("Failed to insert user");

    diesel::sql_query(
        "INSERT INTO sessions (id, user_id, refresh_token, expires_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::VarChar, _>("test-refresh-token")
    .execute(&mut conn)
    .expect("Failed to insert session");

    diesel::sql_query("DELETE FROM users WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .execute(&mut conn)
        .expect("Failed to delete user");

    let count: i64 = sql::<diesel::sql_types::BigInt>(&format!(
        "SELECT COUNT(*) FROM sessions WHERE id = '{}'",
        session_id
    ))
    .get_result(&mut conn)
    .expect("Failed to count sessions");

    assert_eq!(count, 0, "Session should be cascade deleted");
}

#[test]
fn unique_email_constraint() {
    let docker = Cli::default();
    let (_container, pool) = setup(&docker);
    let mut conn = pool.get().expect("Failed to get connection");

    let user_id1 = Uuid::now_v7();
    let user_id2 = Uuid::now_v7();

    diesel::sql_query("INSERT INTO users (id, email, name, password_hash) VALUES ($1, $2, $3, $4)")
        .bind::<diesel::sql_types::Uuid, _>(user_id1)
        .bind::<diesel::sql_types::VarChar, _>("duplicate@example.com")
        .bind::<diesel::sql_types::VarChar, _>("User 1")
        .bind::<diesel::sql_types::VarChar, _>("hashed_password")
        .execute(&mut conn)
        .expect("Failed to insert first user");

    let result = diesel::sql_query(
        "INSERT INTO users (id, email, name, password_hash) VALUES ($1, $2, $3, $4)",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id2)
    .bind::<diesel::sql_types::VarChar, _>("duplicate@example.com")
    .bind::<diesel::sql_types::VarChar, _>("User 2")
    .bind::<diesel::sql_types::VarChar, _>("hashed_password")
    .execute(&mut conn);

    assert!(
        result.is_err(),
        "Should fail with unique constraint violation"
    );
}

#[test]
fn unique_slug_constraint() {
    let docker = Cli::default();
    let (_container, pool) = setup(&docker);
    let mut conn = pool.get().expect("Failed to get connection");

    let user_id1 = Uuid::now_v7();
    let user_id2 = Uuid::now_v7();
    let profile_id1 = Uuid::now_v7();
    let profile_id2 = Uuid::now_v7();

    diesel::sql_query("INSERT INTO users (id, email, name, password_hash) VALUES ($1, $2, $3, $4)")
        .bind::<diesel::sql_types::Uuid, _>(user_id1)
        .bind::<diesel::sql_types::VarChar, _>("user1@example.com")
        .bind::<diesel::sql_types::VarChar, _>("User 1")
        .bind::<diesel::sql_types::VarChar, _>("hashed_password")
        .execute(&mut conn)
        .expect("Failed to insert first user");

    diesel::sql_query("INSERT INTO users (id, email, name, password_hash) VALUES ($1, $2, $3, $4)")
        .bind::<diesel::sql_types::Uuid, _>(user_id2)
        .bind::<diesel::sql_types::VarChar, _>("user2@example.com")
        .bind::<diesel::sql_types::VarChar, _>("User 2")
        .bind::<diesel::sql_types::VarChar, _>("hashed_password")
        .execute(&mut conn)
        .expect("Failed to insert second user");

    diesel::sql_query("INSERT INTO profiles (id, user_id, slug) VALUES ($1, $2, $3)")
        .bind::<diesel::sql_types::Uuid, _>(profile_id1)
        .bind::<diesel::sql_types::Uuid, _>(user_id1)
        .bind::<diesel::sql_types::VarChar, _>("duplicate-slug")
        .execute(&mut conn)
        .expect("Failed to insert first profile");

    let result = diesel::sql_query("INSERT INTO profiles (id, user_id, slug) VALUES ($1, $2, $3)")
        .bind::<diesel::sql_types::Uuid, _>(profile_id2)
        .bind::<diesel::sql_types::Uuid, _>(user_id2)
        .bind::<diesel::sql_types::VarChar, _>("duplicate-slug")
        .execute(&mut conn);

    assert!(
        result.is_err(),
        "Should fail with unique constraint violation"
    );
}

#[test]
fn uuid_v7_is_time_ordered() {
    let uuid1 = Uuid::now_v7();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let uuid2 = Uuid::now_v7();

    assert!(uuid2 > uuid1, "UUID v7 should be time-ordered");
}
