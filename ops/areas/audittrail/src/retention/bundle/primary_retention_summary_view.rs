use super::*;

use acme_kit::text;

/// How screens and exports show a primary retention summary.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionSummaryView {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub timezone_text: String,
    pub score: String,
    pub segment_text: String,
    pub grace_days: String,
    pub summary: String,
    pub scope: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_primary_retention_summary_view(item: &PrimaryRetentionSummary) -> PrimaryRetentionSummaryView {
    PrimaryRetentionSummaryView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        external_ref: item.external_ref.clone(),
        timezone_text: item.timezone.clone(),
        score: format!("{:.2}", item.score),
        segment_text: item.segment.clone(),
        grace_days: item.grace_days.to_string(),
        summary: item.summary.clone(),
        scope: text::truncate(&item.scope, 80),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_retention_summary_views(items: &[PrimaryRetentionSummary]) -> Vec<PrimaryRetentionSummaryView> {
    let mut views: Vec<PrimaryRetentionSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_retention_summary_view(item));
    }
    views
}

pub fn sum_primary_retention_summary_view(view: &PrimaryRetentionSummaryView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_score + (parsed_grace_days as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_retention_summary("code-666", "label-459");
        let view = to_primary_retention_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_retention_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
