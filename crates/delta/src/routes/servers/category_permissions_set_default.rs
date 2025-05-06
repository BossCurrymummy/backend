use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference}, Channel, Database, PartialCategory, PartialChannel, User
};
use revolt_models::v0::{self, DataDefaultChannelPermissions};
use revolt_permissions::{calculate_category_permissions, calculate_channel_permissions, ChannelPermission};
use revolt_result::{create_error, Result};
use rocket::{serde::json::Json, State};

/// # Set Default Permission
///
/// Sets permissions for the default role in this channel.
///
/// Channel must be a `Group`, `TextChannel` or `VoiceChannel`.
#[openapi(tag = "Category Permissions")]
#[put("/<server>/categories/<target>/permissions/default", data = "<data>", rank = 1)]
pub async fn set_default_permissions(
    db: &State<Database>,
    user: User,
    server: Reference,
    target: String,
    data: Json<v0::DataDefaultCategoryPermissions>,
) -> Result<Json<v0::Category>> {
    let data = data.into_inner();

    let mut server = server.as_server(db).await?;
    let mut category = server.categories.get(&target).ok_or(create_error!(UnknownCategory))?.clone();

    let mut query = DatabasePermissionQuery::new(db, &user).server(&server).category(&category);
    let permissions = calculate_category_permissions(&mut query).await;

    permissions.throw_if_lacking_channel_permission(ChannelPermission::ManagePermissions)?;

    permissions
        .throw_permission_override(category.default_permissions.map(|x| x.into()), &data.permissions)
        .await?;

    category
        .update(
            db,
            &mut server,
            PartialCategory {
                default_permissions: Some(data.permissions.into()),
                ..Default::default()
            },
            vec![],
        )
        .await?;

    Ok(Json(category.into()))
}
