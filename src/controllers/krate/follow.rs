//! Endpoints for managing a per user list of followed crates

use crate::auth::AuthCheck;
use diesel::associations::Identifiable;

use crate::controllers::frontend_prelude::*;
use crate::models::{Crate, Follow};
use crate::schema::*;
use crate::util::errors::crate_not_found;

fn follow_target(crate_name: &str, conn: &mut PgConnection, user_id: i32) -> AppResult<Follow> {
    let crate_id = Crate::by_name(crate_name)
        .select(crates::id)
        .first(conn)
        .optional()?
        .ok_or_else(|| crate_not_found(crate_name))?;

    Ok(Follow { user_id, crate_id })
}

/// Handles the `PUT /crates/:crate_id/follow` route.
pub async fn follow(
    app: AppState,
    Path(crate_name): Path<String>,
    req: Parts,
) -> AppResult<Response> {
    spawn_blocking(move || {
        let conn = &mut *app.db_write()?;
        let user_id = AuthCheck::default().check(&req, conn)?.user_id();
        let follow = follow_target(&crate_name, conn, user_id)?;
        diesel::insert_into(follows::table)
            .values(&follow)
            .on_conflict_do_nothing()
            .execute(conn)?;

        ok_true()
    })
    .await
}

/// Handles the `DELETE /crates/:crate_id/follow` route.
pub async fn unfollow(
    app: AppState,
    Path(crate_name): Path<String>,
    req: Parts,
) -> AppResult<Response> {
    spawn_blocking(move || {
        let conn = &mut *app.db_write()?;
        let user_id = AuthCheck::default().check(&req, conn)?.user_id();
        let follow = follow_target(&crate_name, conn, user_id)?;
        diesel::delete(&follow).execute(conn)?;

        ok_true()
    })
    .await
}

/// Handles the `GET /crates/:crate_id/following` route.
pub async fn following(
    app: AppState,
    Path(crate_name): Path<String>,
    req: Parts,
) -> AppResult<Json<Value>> {
    spawn_blocking(move || {
        use diesel::dsl::exists;

        let conn = &mut *app.db_read_prefer_primary()?;
        let user_id = AuthCheck::only_cookie().check(&req, conn)?.user_id();
        let follow = follow_target(&crate_name, conn, user_id)?;
        let following =
            diesel::select(exists(follows::table.find(follow.id()))).get_result::<bool>(conn)?;

        Ok(Json(json!({ "following": following })))
    })
    .await
}
