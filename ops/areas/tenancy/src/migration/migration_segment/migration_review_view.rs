use super::*;

use acme_kit::text;

/// How screens and exports show a migration review.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationReviewView {
    pub code_text: String,
    pub label: String,
    pub sequence_text: String,
    pub channel_text: String,
    pub summary_text: String,
    pub audience: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_migration_review_view(item: &MigrationReview) -> MigrationReviewView {
    MigrationReviewView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        sequence_text: item.sequence.to_string(),
        channel_text: item.channel.clone(),
        summary_text: item.summary.clone(),
        audience: text::truncate(&item.audience, 40),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_review_views(items: &[MigrationReview]) -> Vec<MigrationReviewView> {
    let mut views: Vec<MigrationReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_review_view(item));
    }
    views
}

pub fn sum_migration_review_view(view: &MigrationReviewView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_sequence as f64)
}
