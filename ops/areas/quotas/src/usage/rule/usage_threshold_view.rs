use super::*;

use acme_kit::text;

/// How screens and exports show a usage threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageThresholdView {
    pub code_text: String,
    pub title_text: String,
    pub budget_text: String,
    pub cluster: String,
    pub notes: String,
    pub tenant_id: String,
    pub country_text: String,
    pub margin: String,
    pub checksum_text: String,
    pub status_label: String,
}

pub fn to_usage_threshold_view(item: &UsageThreshold) -> UsageThresholdView {
    UsageThresholdView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        budget_text: format!("{:.2}", item.budget),
        cluster: item.cluster.clone(),
        notes: item.notes.clone(),
        tenant_id: item.tenant_id.clone(),
        country_text: item.country.clone(),
        margin: format!("{:.2}", item.margin),
        checksum_text: text::truncate(&item.checksum, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_threshold_views(items: &[UsageThreshold]) -> Vec<UsageThresholdView> {
    let mut views: Vec<UsageThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_threshold_view(item));
    }
    views
}

pub fn sum_usage_threshold_view(view: &UsageThresholdView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_budget + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_threshold("code-103", "title-531");
        let view = to_usage_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
