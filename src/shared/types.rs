pub type DbResult<T> = Result<T, diesel::result::Error>;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub token: String,
}
