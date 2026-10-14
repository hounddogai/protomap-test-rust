use super::*;

use acme_kit::text;

/// How screens and exports show a failover rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRollupView {
    pub code: String,
    pub label: String,
    pub title_text: String,
    pub approved: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_failover_rollup_view(item: &FailoverRollup) -> FailoverRollupView {
    FailoverRollupView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        title_text: item.title.clone(),
        approved: format!("{}", item.approved),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_rollup_views(items: &[FailoverRollup]) -> Vec<FailoverRollupView> {
    let mut views: Vec<FailoverRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_rollup_view(item));
    }
    views
}

pub fn sum_failover_rollup_view(view: &FailoverRollupView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_quota_units as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_rollup("code-57", "label-103");
        let view = to_failover_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
