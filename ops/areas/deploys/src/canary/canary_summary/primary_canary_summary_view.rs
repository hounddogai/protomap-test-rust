use super::*;

use acme_kit::text;

/// How screens and exports show a primary canary summary.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanarySummaryView {
    pub code: String,
    pub label_text: String,
    pub tenant_id: String,
    pub region: String,
    pub revision_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_primary_canary_summary_view(item: &PrimaryCanarySummary) -> PrimaryCanarySummaryView {
    PrimaryCanarySummaryView {
        code: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        tenant_id: text::truncate(&item.tenant_id, 120),
        region: item.region.clone(),
        revision_text: item.revision.to_string(),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_canary_summary_views(items: &[PrimaryCanarySummary]) -> Vec<PrimaryCanarySummaryView> {
    let mut views: Vec<PrimaryCanarySummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_canary_summary_view(item));
    }
    views
}

pub fn sum_primary_canary_summary_view(view: &PrimaryCanarySummaryView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_revision as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_canary_summary("code-605", "label-764");
        let view = to_primary_canary_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_canary_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
