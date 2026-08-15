use crate::shared::db::DbPool;
use crate::waitlist::repository::DieselWaitlistRepository;
use crate::waitlist::service::WaitlistService;

#[derive(Clone)]
pub struct AppState {
    pub waitlist_service: WaitlistService<DieselWaitlistRepository>,
}

impl AppState {
    pub fn new(pool: DbPool) -> Self {
        let waitlist_repo = DieselWaitlistRepository::new(pool);
        let waitlist_service = WaitlistService::new(waitlist_repo);

        Self { waitlist_service }
    }
}
