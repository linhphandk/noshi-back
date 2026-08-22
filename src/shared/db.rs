use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use r2d2::event::{
    AcquireEvent, CheckinEvent, CheckoutEvent, HandleEvent, ReleaseEvent, TimeoutEvent,
};
use tracing::{debug, info, warn};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[derive(Debug)]
struct TracingEventHandler;

impl HandleEvent for TracingEventHandler {
    fn handle_acquire(&self, _event: AcquireEvent) {
        debug!("db: connection acquired");
    }

    fn handle_release(&self, _event: ReleaseEvent) {
        debug!("db: connection released");
    }

    fn handle_checkout(&self, event: CheckoutEvent) {
        debug!(
            id = event.connection_id(),
            duration_ms = event.duration().as_millis() as u64,
            "db: connection checked out"
        );
    }

    fn handle_checkin(&self, event: CheckinEvent) {
        debug!(
            id = event.connection_id(),
            duration_ms = event.duration().as_millis() as u64,
            "db: connection returned"
        );
    }

    fn handle_timeout(&self, event: TimeoutEvent) {
        warn!(
            timeout_ms = event.timeout().as_millis() as u64,
            "db: connection acquisition timed out"
        );
    }
}

pub fn establish_pool(database_url: &str, pool_size: u32) -> DbPool {
    debug!(pool_size, "establishing connection pool");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(pool_size)
        .event_handler(Box::new(TracingEventHandler))
        .build(manager)
        .expect("Failed to create database pool");
    info!(pool_size, "database pool ready");
    pool
}

pub fn run_migrations(pool: &DbPool) {
    info!("running pending migrations");
    let mut conn = pool.get().expect("Failed to get connection");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
    info!("migrations complete");
}
