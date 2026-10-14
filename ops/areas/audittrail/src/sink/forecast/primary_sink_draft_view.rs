use super::*;

use acme_kit::text;

/// How screens and exports show a primary sink draft.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkDraftView {
    pub code: String,
    pub name: String,
    pub ttl_seconds: String,
    pub notes: String,
    pub cluster_text: String,
    pub quantity_text: String,
    pub reviewer_text: String,
    pub ratio: String,
    pub limit: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_primary_sink_draft_view(item: &PrimarySinkDraft) -> PrimarySinkDraftView {
    PrimarySinkDraftView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        notes: item.notes.clone(),
        cluster_text: item.cluster.clone(),
        quantity_text: item.quantity.to_string(),
        reviewer_text: text::truncate(&item.reviewer, 120),
        ratio: format!("{:.2}", item.ratio),
        limit: item.limit.to_string(),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_sink_draft_views(items: &[PrimarySinkDraft]) -> Vec<PrimarySinkDraftView> {
    let mut views: Vec<PrimarySinkDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_sink_draft_view(item));
    }
    views
}

pub fn sum_primary_sink_draft_view(view: &PrimarySinkDraftView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_quantity as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_sink_draft("code-393", "name-511");
        let view = to_primary_sink_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_sink_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
