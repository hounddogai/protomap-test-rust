use super::*;

use acme_kit::text;

/// How screens and exports show a health check rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRollupView {
    pub code: String,
    pub name_text: String,
    pub reviewer_text: String,
    pub external_ref_text: String,
    pub archived_text: String,
    pub grace_days: String,
    pub maximum: String,
    pub summary_text: String,
    pub status_label: String,
}

pub fn to_health_check_rollup_view(item: &HealthCheckRollup) -> HealthCheckRollupView {
    HealthCheckRollupView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        reviewer_text: text::truncate(&item.reviewer, 40),
        external_ref_text: item.external_ref.clone(),
        archived_text: format!("{}", item.archived),
        grace_days: item.grace_days.to_string(),
        maximum: format!("{:.2}", item.maximum),
        summary_text: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_rollup_views(items: &[HealthCheckRollup]) -> Vec<HealthCheckRollupView> {
    let mut views: Vec<HealthCheckRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_rollup_view(item));
    }
    views
}

pub fn sum_health_check_rollup_view(view: &HealthCheckRollupView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_rollup("code-828", "name-628");
        let view = to_health_check_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
