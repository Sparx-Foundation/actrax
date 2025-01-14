use sqlx::{PgPool, Row};

pub async fn initialize_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    // TODO: fix this shit

    sqlx::query(include_str!("../../../migrations/user/0001_client_table.sql")).execute(pool).await.map_err(|e| {
        sqlx::Error::from(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to execute schema: {}", e),
        ))
    })?;

    Ok(())
}


#[allow(unused)]
pub async fn search_client(
    pool: &PgPool,
    uid: Option<&str>,
    id: Option<i32>,
    name: Option<&str>,
    partial: bool
) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
    let mut query = String::from("SELECT * FROM actrax_client WHERE 1=1");

    if let Some(_uid) = uid {
        if partial {
            query.push_str(" AND uid ILIKE $1");
        } else {
            query.push_str(" AND uid = $1");
        }
    }
    if let Some(_id) = id {
        query.push_str(" AND id = $2");
    }
    if let Some(_name) = name {
        if partial {
            query.push_str(" AND name ILIKE $3");
        } else {
            query.push_str(" AND name = $3");
        }
    }

    sqlx::query(&query)
        .bind(uid.unwrap_or(""))
        .bind(id.unwrap_or(0))
        .bind(name.unwrap_or(""))
        .fetch_all(pool)
        .await
}

#[allow(unused)]
pub async fn create_client(pool: &PgPool, uid: &str, name: Option<&str>) -> Result<i32, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO actrax_client (uid, name)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind(uid)
    .bind(name)
    .fetch_one(pool)
    .await?;

    let user_id: i32 = row.get("id");
    Ok(user_id)
}

#[allow(unused)]
pub async fn update_client(
    pool: &PgPool,
    id: i32,
    new_uid: Option<&str>,
    new_name: Option<&str>
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE actrax_client
        SET
            uid = COALESCE($1, uid),
            name = COALESCE($2, name)
        WHERE id = $3
        "#,
    )
    .bind(new_uid)
    .bind(new_name)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

#[allow(unused)]
pub async fn delete_client(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM actrax_client WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
