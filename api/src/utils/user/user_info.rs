use anyhow::{anyhow, Result};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::{
    lib::get_pg_pool,
    models::Organization,
    schema::{organizations, users_to_organizations},
};

pub async fn get_user_organization_id(user_id: &Uuid) -> Result<Uuid> {
    let mut conn = get_pg_pool().get().await?;

    let organization_id = match users_to_organizations::table
        .select(users_to_organizations::organization_id)
        .filter(users_to_organizations::user_id.eq(user_id))
        .filter(users_to_organizations::deleted_at.is_null())
        .first::<Uuid>(&mut conn)
        .await
    {
        Ok(organization_id) => organization_id,
        Err(diesel::NotFound) => return Err(anyhow!("User not found")),
        Err(e) => return Err(anyhow!("Error getting user organization id: {}", e)),
    };

    Ok(organization_id)
}

pub async fn get_user_organization(user_id: &Uuid) -> Result<Organization> {
    let mut conn = get_pg_pool().get().await?;

    let organization = match organizations::table
        .inner_join(
            users_to_organizations::table.on(users_to_organizations::organization_id
                .eq(organizations::id)
                .and(
                    users_to_organizations::user_id
                        .eq(user_id)
                        .and(users_to_organizations::deleted_at.is_null()),
                )),
        )
        .select(organizations::all_columns)
        .filter(users_to_organizations::user_id.eq(user_id))
        .filter(users_to_organizations::deleted_at.is_null())
        .first::<Organization>(&mut conn)
        .await
    {
        Ok(organization) => organization,
        Err(diesel::NotFound) => return Err(anyhow!("Organization not found.")),
        Err(e) => return Err(anyhow!("Error getting user organization id: {}", e)),
    };

    Ok(organization)
}
