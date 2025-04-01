pub async fn query_safety_filter(sql: String) -> Option<String> {
    let uppercase_sql = sql.to_uppercase();

    if uppercase_sql.contains("INFORMATION_SCHEMA") {
        return Some("Access denied to information_schema.".to_string());
    }

    if uppercase_sql.contains("UPDATE ") {
        return Some(
            "I'm not allowed to update the database. Please try another request.".to_string(),
        );
    }

    if uppercase_sql.contains("DELETE ") {
        return Some(
            "I'm not allowed to delete from the database. Please try another request.".to_string(),
        );
    }

    if uppercase_sql.contains("INSERT ") {
        return Some(
            "I'm not allowed to insert into the database. Please try another request.".to_string(),
        );
    }

    if uppercase_sql.contains("DROP ") {
        return Some(
            "I'm not allowed to drop tables in the database. Please try another request."
                .to_string(),
        );
    }

    if uppercase_sql.contains("CREATE ") {
        return Some(
            "I'm not allowed to create tables in the database. Please try another request."
                .to_string(),
        );
    }

    if uppercase_sql.contains("ALTER ") {
        return Some(
            "I'm not allowed to alter tables in the database. Please try another request."
                .to_string(),
        );
    }

    if uppercase_sql.contains("GRANT ") {
        return Some(
            "I'm not allowed to grant permissions in the database. Please try another request."
                .to_string(),
        );
    }

    if uppercase_sql.contains("REVOKE ") {
        return Some(
            "I'm not allowed to revoke permissions in the database. Please try another request."
                .to_string(),
        );
    }

    None
}

pub async fn write_query_safety_filter(sql: String) -> Option<String> {
    let uppercase_sql = sql.to_uppercase();

    if !(uppercase_sql.contains("CREATE VIEW")
        || uppercase_sql.contains("CREATE MATERIALIZED VIEW")
        || uppercase_sql.contains("DROP VIEW IF EXISTS")
        || uppercase_sql.contains("DROP MATERIALIZED VIEW IF EXISTS")
        || uppercase_sql.contains("CREATE OR REPLACE VIEW")
        || uppercase_sql.contains("CREATE MATERIALIZED VIEW IF NOT EXISTS"))
    {
        return Some(
            "I'm only allowed to create, replace, or drop views or materialized views. Please try another request."
                .to_string(),
        );
    }

    if uppercase_sql.contains("DELETE ") {
        return Some(
            "I'm not allowed to delete from the database. Please try another request.".to_string(),
        );
    }

    if uppercase_sql.contains("INSERT ") {
        return Some(
            "I'm not allowed to insert into the database. Please try another request.".to_string(),
        );
    }

    if uppercase_sql.contains("ALTER ") {
        return Some(
            "I'm not allowed to alter tables in the database. Please try another request."
                .to_string(),
        );
    }

    if uppercase_sql.contains("GRANT ") {
        return Some(
            "I'm not allowed to grant permissions in the database. Please try another request."
                .to_string(),
        );
    }

    if uppercase_sql.contains("REVOKE ") {
        return Some(
            "I'm not allowed to revoke permissions in the database. Please try another request."
                .to_string(),
        );
    }

    None
}
