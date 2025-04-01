use crate::database::lib::get_pg_pool;
use crate::database::schema::{datasets, messages};
use crate::routes::rest::ApiResponse;
use crate::utils::clients::sentry_utils::send_sentry_error;
use crate::utils::clients::typesense::{
    upsert_document, CollectionName, Document, GenericDocument, MessageDocument,
};
use axum::http::StatusCode;
use axum::Json;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct EmbeddingRequest {
    table: CollectionName,
    record: serde_json::Value,
}

pub async fn create_record_embedding(
    Json(req): Json<EmbeddingRequest>,
) -> Result<ApiResponse<Vec<f32>>, (StatusCode, &'static str)> {
    tokio::spawn(async move {
        match update_record(req.table, req.record).await {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Error while updating the record: {:?}", e);
                send_sentry_error(&e.to_string(), None);
            }
        }
    });

    Ok(ApiResponse::OK)
}

async fn update_record(table: CollectionName, record: Value) -> Result<(), anyhow::Error> {
    let pg_pool = get_pg_pool();

    let id = match record.get("id") {
        Some(id) => Uuid::parse_str(id.as_str().unwrap_or_default()).unwrap(),
        None => return Ok(()),
    };

    let name = match table {
        CollectionName::Messages => match record.get("title") {
            Some(title) => title.as_str().unwrap_or_default(),
            None => {
                tracing::debug!("Title is not present in the record");
                return Ok(());
            }
        },
        _ => match record.get("name") {
            Some(name) => name.as_str().unwrap_or_default(),
            None => {
                tracing::debug!("Name is not present in the record");
                return Ok(());
            }
        },
    };

    let mut conn = pg_pool.get().await?;

    let organization_id = match table {
        CollectionName::Messages => {
            match messages::table
                .inner_join(datasets::table)
                .select(datasets::organization_id)
                .filter(messages::id.eq(id))
                .first::<Uuid>(&mut conn)
                .await
            {
                Ok(organization_id) => organization_id,
                Err(e) => {
                    tracing::debug!("Error while getting the organization id: {:?}", e);
                    return Ok(());
                }
            }
        }
        _ => {
            let organization_id = match record.get("organization_id") {
                Some(organization_id) => {
                    Uuid::parse_str(organization_id.as_str().unwrap_or_default()).unwrap()
                }
                None => {
                    tracing::debug!("Organization id is not present in the record");
                    return Ok(());
                }
            };

            organization_id
        }
    };

    let document = match table {
        CollectionName::Messages => {
            let summary_question = match record.get("summary_question") {
                Some(summary_question) => summary_question.as_str().unwrap_or_default(),
                None => {
                    tracing::debug!("Summary question is not present in the record");
                    return Ok(());
                }
            };

            let message_upsert = MessageDocument {
                id,
                name: name.replace("\"", ""),
                summary_question: summary_question.replace("\"", ""),
                organization_id,
            };

            Document::Message(message_upsert)
        }
        CollectionName::Collections => {
            let collection_upsert = GenericDocument {
                id,
                name: name.replace("\"", ""),
                organization_id,
            };

            Document::Collection(collection_upsert)
        }
        CollectionName::Datasets => {
            let dataset_upsert = GenericDocument {
                id,
                name: name.replace("\"", ""),
                organization_id,
            };

            Document::Dataset(dataset_upsert)
        }
        CollectionName::PermissionGroups => {
            let permission_group_upsert = GenericDocument {
                id,
                name: name.replace("\"", ""),
                organization_id,
            };

            Document::PermissionGroup(permission_group_upsert)
        }
        CollectionName::Teams => {
            let team_upsert = GenericDocument {
                id,
                name: name.replace("\"", ""),
                organization_id,
            };

            Document::Team(team_upsert)
        }
        CollectionName::DataSources => {
            let data_source_upsert = GenericDocument {
                id,
                name: name.replace("\"", ""),
                organization_id,
            };

            Document::DataSource(data_source_upsert)
        }
        CollectionName::Terms => {
            let term_upsert = GenericDocument {
                id,
                name: name.replace("\"", ""),
                organization_id,
            };

            Document::Term(term_upsert)
        }
        CollectionName::Dashboards => {
            let dashboard_upsert = GenericDocument {
                id,
                name: name.replace("\"", ""),
                organization_id,
            };

            Document::Dashboard(dashboard_upsert)
        }
        _ => return Ok(()),
    };

    match upsert_document(table, document).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error while upserting the document: {:?}", e);
            return Err(e);
        }
    }

    Ok(())
}
