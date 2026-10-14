use super::*;

use acme_kit::text;

/// How screens and exports show a grant summary.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantSummaryView {
    pub code_text: String,
    pub label_text: String,
    pub category: String,
    pub unit: String,
    pub timezone: String,
    pub issuer: String,
    pub namespace: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_grant_summary_view(item: &GrantSummary) -> GrantSummaryView {
    GrantSummaryView {
        code_text: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        category: item.category.clone(),
        unit: text::truncate(&item.unit, 120),
        timezone: item.timezone.clone(),
        issuer: item.issuer.clone(),
        namespace: text::truncate(&item.namespace, 80),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_summary_views(items: &[GrantSummary]) -> Vec<GrantSummaryView> {
    let mut views: Vec<GrantSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_summary_view(item));
    }
    views
}

pub fn sum_grant_summary_view(view: &GrantSummaryView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_summary("code-962", "label-254");
        let view = to_grant_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
