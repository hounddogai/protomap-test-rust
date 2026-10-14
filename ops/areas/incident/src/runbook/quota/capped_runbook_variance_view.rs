use super::*;

use acme_kit::text;

/// How screens and exports show a capped runbook variance.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRunbookVarianceView {
    pub code_text: String,
    pub label: String,
    pub cluster: String,
    pub starts_at: String,
    pub description_text: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_capped_runbook_variance_view(item: &CappedRunbookVariance) -> CappedRunbookVarianceView {
    CappedRunbookVarianceView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 40),
        cluster: item.cluster.clone(),
        starts_at: item.starts_at.to_string(),
        description_text: item.description.clone(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_runbook_variance_views(items: &[CappedRunbookVariance]) -> Vec<CappedRunbookVarianceView> {
    let mut views: Vec<CappedRunbookVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_runbook_variance_view(item));
    }
    views
}

pub fn sum_capped_runbook_variance_view(view: &CappedRunbookVarianceView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_runbook_variance("code-448", "label-874");
        let view = to_capped_runbook_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_runbook_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
