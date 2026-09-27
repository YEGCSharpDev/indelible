//! `FilterNode` to SQL for saved Library content.

use sqlx::{Postgres, QueryBuilder};

use ind_application::AppError;
use ind_domain::{ALLOWED_FILTER_FIELDS, FilterNode, FilterOp};

use crate::repos::filter_sql::{
    parse_prefixed_uuid, push_boolean_filter, push_text_filter, push_timestamp_filter,
    validation_error,
};

pub(super) fn push_filter_node(
    builder: &mut QueryBuilder<'_, Postgres>,
    node: &FilterNode,
) -> Result<(), AppError> {
    match node {
        FilterNode::And { conditions } => {
            if conditions.is_empty() {
                builder.push("TRUE");
                return Ok(());
            }
            builder.push("(");
            for (index, condition) in conditions.iter().enumerate() {
                if index > 0 {
                    builder.push(" AND ");
                }
                push_filter_node(builder, condition)?;
            }
            builder.push(")");
        }
        FilterNode::Or { conditions } => {
            if conditions.is_empty() {
                builder.push("FALSE");
                return Ok(());
            }
            builder.push("(");
            for (index, condition) in conditions.iter().enumerate() {
                if index > 0 {
                    builder.push(" OR ");
                }
                push_filter_node(builder, condition)?;
            }
            builder.push(")");
        }
        FilterNode::Not { condition } => {
            builder.push("NOT (");
            push_filter_node(builder, condition)?;
            builder.push(")");
        }
        FilterNode::Condition { field, op, value } => {
            push_filter_condition(builder, field, op, value)?;
        }
    }

    Ok(())
}

fn push_filter_condition(
    builder: &mut QueryBuilder<'_, Postgres>,
    field: &str,
    op: &FilterOp,
    value: &serde_json::Value,
) -> Result<(), AppError> {
    if !ALLOWED_FILTER_FIELDS.contains(&field) {
        return Err(validation_error(&format!(
            "unsupported filter field: {field}"
        )));
    }

    match field {
        "tag" => push_tag_filter(builder, op, value),
        "collection" => push_collection_filter(builder, op, value),
        "is_favorite" => push_boolean_filter(builder, "le.is_favorite", field, op, value),
        "item_type" => push_text_filter(builder, "d.document_type", field, op, value, false),
        "triage_state" => push_text_filter(builder, "le.triage_state", field, op, value, false),
        // domain accepts eq/neq/in only (matches the validator and FE field defs); contains is
        // rejected here so the evaluator and validator agree on the operator matrix.
        "domain" => {
            reject_contains(field, op)?;
            push_text_filter(builder, "d.domain", field, op, value, true)
        }
        "saved_at" => push_timestamp_filter(builder, "le.saved_at", field, op, value),
        "published_at" => push_timestamp_filter(builder, "d.published_at", field, op, value),
        _ => Err(validation_error(&format!("unsupported field: {field}"))),
    }
}

fn reject_contains(field: &str, op: &FilterOp) -> Result<(), AppError> {
    if matches!(op, FilterOp::Contains) {
        return Err(validation_error(&format!(
            "{field} only supports eq, neq, in"
        )));
    }
    Ok(())
}

fn push_tag_filter(
    builder: &mut QueryBuilder<'_, Postgres>,
    op: &FilterOp,
    value: &serde_json::Value,
) -> Result<(), AppError> {
    let tag_name = value
        .as_str()
        .ok_or_else(|| validation_error("tag value must be a string"))?
        .to_string();

    match op {
        FilterOp::Eq | FilterOp::Contains => {
            builder.push(
                "EXISTS (SELECT 1 FROM library_entry_tags let_filter \
                 JOIN tags t_filter ON t_filter.id = let_filter.tag_id \
                 WHERE let_filter.library_entry_id = le.id \
                 AND t_filter.user_id = le.user_id \
                 AND LOWER(t_filter.name) = LOWER(",
            );
            builder.push_bind(tag_name);
            builder.push("))");
            Ok(())
        }
        FilterOp::Neq => {
            builder.push(
                "NOT EXISTS (SELECT 1 FROM library_entry_tags let_filter \
                 JOIN tags t_filter ON t_filter.id = let_filter.tag_id \
                 WHERE let_filter.library_entry_id = le.id \
                 AND t_filter.user_id = le.user_id \
                 AND LOWER(t_filter.name) = LOWER(",
            );
            builder.push_bind(tag_name);
            builder.push("))");
            Ok(())
        }
        _ => Err(validation_error(
            "tag field only supports eq, neq, contains",
        )),
    }
}

fn push_collection_filter(
    builder: &mut QueryBuilder<'_, Postgres>,
    op: &FilterOp,
    value: &serde_json::Value,
) -> Result<(), AppError> {
    let raw_collection_id = value
        .as_str()
        .ok_or_else(|| validation_error("collection value must be a string (collection ID)"))?;
    let collection_id = parse_prefixed_uuid(raw_collection_id, "col_")
        .map_err(|_| validation_error("collection value is not a valid UUID"))?;

    match op {
        FilterOp::Eq | FilterOp::Contains => {
            builder.push(
                "EXISTS (SELECT 1 FROM collection_entries ce_filter \
                 WHERE ce_filter.library_entry_id = le.id \
                 AND ce_filter.user_id = le.user_id \
                 AND ce_filter.collection_id = ",
            );
            builder.push_bind(collection_id);
            builder.push(")");
            Ok(())
        }
        _ => Err(validation_error(
            "collection field only supports eq, contains",
        )),
    }
}
