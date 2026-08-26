use noshi_back::shared::db::{establish_pool, run_migrations};
use noshi_back::waitlist::models::NewWaitlistEntry;
use noshi_back::waitlist::ports::WaitlistRepository;
use noshi_back::waitlist::repository::DieselWaitlistRepository;
use std::sync::OnceLock;
use testcontainers::clients::Cli;
use testcontainers_modules::postgres::Postgres;

fn docker() -> &'static Cli {
    static DOCKER: OnceLock<Cli> = OnceLock::new();
    DOCKER.get_or_init(Cli::default)
}

struct TestDb {
    _container: testcontainers::Container<'static, Postgres>,
    repo: DieselWaitlistRepository,
}

fn setup() -> TestDb {
    let container = docker().run(Postgres::default());
    let port = container.get_host_port_ipv4(5432);
    let db_url = format!("postgres://postgres:postgres@localhost:{}/postgres", port);
    let pool = establish_pool(&db_url, 4);
    run_migrations(&pool);
    TestDb {
        _container: container,
        repo: DieselWaitlistRepository::new(pool),
    }
}

#[tokio::test]
async fn test_create_and_find_by_email() {
    let db = setup();
    let email = format!("user-{}@test.com", uuid::Uuid::now_v7());

    let entry = db
        .repo
        .create(NewWaitlistEntry {
            email: email.clone(),
        })
        .await
        .unwrap();

    assert_eq!(entry.email, email);

    let found = db.repo.find_by_email(&email).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().email, email);
}

#[tokio::test]
async fn test_find_by_email_nonexistent() {
    let db = setup();
    let found = db.repo.find_by_email("nonexistent@test.com").await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_duplicate_email_returns_error() {
    let db = setup();
    let email = format!("dup-{}@test.com", uuid::Uuid::now_v7());

    db.repo
        .create(NewWaitlistEntry {
            email: email.clone(),
        })
        .await
        .unwrap();

    let result = db
        .repo
        .create(NewWaitlistEntry { email })
        .await;

    assert!(result.is_err());
}
