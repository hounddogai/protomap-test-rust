use super::*;

use acme_kit::text;

/// How screens and exports show a regional data export review.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReviewView {
    pub code: String,
    pub name_text: String,
    pub version: String,
    pub notes: String,
    pub audience: String,
    pub amount_text: String,
    pub pinned: String,
    pub contact_email_text: String,
    pub status_label: String,
}

pub fn to_regional_data_export_review_view(item: &RegionalDataExportReview) -> RegionalDataExportReviewView {
    RegionalDataExportReviewView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        version: item.version.to_string(),
        notes: text::truncate(&item.notes, 40),
        audience: item.audience.clone(),
        amount_text: format!("{:.2}", item.amount),
        pinned: format!("{}", item.pinned),
        contact_email_text: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_data_export_review_views(items: &[RegionalDataExportReview]) -> Vec<RegionalDataExportReviewView> {
    let mut views: Vec<RegionalDataExportReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_data_export_review_view(item));
    }
    views
}

pub fn sum_regional_data_export_review_view(view: &RegionalDataExportReviewView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_version as f64 + parsed_amount)
}
