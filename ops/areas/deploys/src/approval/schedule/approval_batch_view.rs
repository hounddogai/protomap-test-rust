use super::*;

use acme_kit::text;

/// How screens and exports show a approval batch.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBatchView {
    pub code: String,
    pub name: String,
    pub description: String,
    pub discount: String,
    pub phone: String,
    pub limit: String,
    pub maximum: String,
    pub category: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_approval_batch_view(item: &ApprovalBatch) -> ApprovalBatchView {
    ApprovalBatchView {
        code: item.code.clone(),
        name: item.name.clone(),
        description: item.description.clone(),
        discount: format!("{:.2}", item.discount),
        phone: text::truncate(&item.phone, 64),
        limit: item.limit.to_string(),
        maximum: format!("{:.2}", item.maximum),
        category: item.category.clone(),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_batch_views(items: &[ApprovalBatch]) -> Vec<ApprovalBatchView> {
    let mut views: Vec<ApprovalBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_batch_view(item));
    }
    views
}

pub fn sum_approval_batch_view(view: &ApprovalBatchView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_discount + (parsed_limit as f64) + parsed_maximum)
}
