use crate::database::{DbPool, get_db_conn};
use crate::models::users::{NewUser, User};
use crate::schema::users::dsl::*;
use diesel::prelude::*;

/// Inserts a new user and returns the created user
pub fn insert_user(pool: &DbPool, new_user: &NewUser) -> Result<User, diesel::result::Error> {
    let mut conn = get_db_conn(pool)?;

    diesel::insert_into(users)
        .values(new_user)
        .get_result(&mut conn)
}

/// Finds a user by oauth provider and oauth user id.
pub fn find_user_by_oauth(
    pool: &DbPool,
    provider: &str,
    oauth_id: &str,
) -> Result<Option<User>, diesel::result::Error> {
    let mut conn = get_db_conn(pool)?;

    let user_opt = users
        .filter(oauth_provider.eq(provider))
        .filter(oauth_user_id.eq(oauth_id))
        .first::<User>(&mut conn)
        .optional()?;

    Ok(user_opt)
}
