use super::*;

use acme_kit::text;

/// How screens and exports show a allowlist summary.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSummaryView {
    pub code: String,
    pub name_text: String,
    pub country: String,
    pub phone_text: String,
    pub cluster: String,
    pub source: String,
    pub score: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_allowlist_summary_view(item: &AllowlistSummary) -> AllowlistSummaryView {
    AllowlistSummaryView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        country: text::truncate(&item.country, 120),
        phone_text: item.phone.clone(),
        cluster: item.cluster.clone(),
        source: item.source.clone(),
        score: format!("{:.2}", item.score),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_summary_views(items: &[AllowlistSummary]) -> Vec<AllowlistSummaryView> {
    let mut views: Vec<AllowlistSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_summary_view(item));
    }
    views
}

pub fn sum_allowlist_summary_view(view: &AllowlistSummaryView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_allowlist_summary("code-954", "name-727");
        let view = to_allowlist_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_allowlist_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
