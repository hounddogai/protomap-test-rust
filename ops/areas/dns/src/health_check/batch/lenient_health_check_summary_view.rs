use super::*;

use acme_kit::text;

/// How screens and exports show a lenient health check summary.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientHealthCheckSummaryView {
    pub code_text: String,
    pub name: String,
    pub limit: String,
    pub fingerprint: String,
    pub archived: String,
    pub created_at: String,
    pub visible: String,
    pub expires_at_text: String,
    pub weight: String,
    pub country: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_lenient_health_check_summary_view(item: &LenientHealthCheckSummary) -> LenientHealthCheckSummaryView {
    LenientHealthCheckSummaryView {
        code_text: text::truncate(&item.code, 80),
        name: item.name.clone(),
        limit: item.limit.to_string(),
        fingerprint: item.fingerprint.clone(),
        archived: format!("{}", item.archived),
        created_at: item.created_at.to_string(),
        visible: format!("{}", item.visible),
        expires_at_text: item.expires_at.to_string(),
        weight: format!("{:.2}", item.weight),
        country: item.country.clone(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_health_check_summary_views(
    items: &[LenientHealthCheckSummary],
) -> Vec<LenientHealthCheckSummaryView> {
    let mut views: Vec<LenientHealthCheckSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_health_check_summary_view(item));
    }
    views
}

pub fn sum_lenient_health_check_summary_view(view: &LenientHealthCheckSummaryView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_created_at as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_health_check_summary("code-361", "name-338");
        let view = to_lenient_health_check_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_health_check_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
