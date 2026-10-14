use super::*;

use acme_kit::text;

/// How screens and exports show a rolling page summary.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageSummaryView {
    pub code: String,
    pub label: String,
    pub ceiling: String,
    pub category: String,
    pub maximum_text: String,
    pub tenant_id_text: String,
    pub attempts_text: String,
    pub namespace_text: String,
    pub cipher: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_rolling_page_summary_view(item: &RollingPageSummary) -> RollingPageSummaryView {
    RollingPageSummaryView {
        code: item.code.clone(),
        label: item.label.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        category: text::truncate(&item.category, 64),
        maximum_text: format!("{:.2}", item.maximum),
        tenant_id_text: item.tenant_id.clone(),
        attempts_text: item.attempts.to_string(),
        namespace_text: text::truncate(&item.namespace, 40),
        cipher: item.cipher.clone(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_page_summary_views(items: &[RollingPageSummary]) -> Vec<RollingPageSummaryView> {
    let mut views: Vec<RollingPageSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_page_summary_view(item));
    }
    views
}

pub fn sum_rolling_page_summary_view(view: &RollingPageSummaryView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_ceiling + parsed_maximum + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_page_summary("code-400", "label-564");
        let view = to_rolling_page_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_page_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
