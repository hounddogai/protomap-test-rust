use super::*;

use acme_kit::text;

/// How screens and exports show a strict dashboard override.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardOverrideView {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub description: String,
    pub host: String,
    pub region_text: String,
    pub batch_size: String,
    pub audience_text: String,
    pub attempts_text: String,
    pub contact_email: String,
    pub timezone_text: String,
    pub status_label: String,
}

pub fn to_strict_dashboard_override_view(item: &StrictDashboardOverride) -> StrictDashboardOverrideView {
    StrictDashboardOverrideView {
        code: item.code.clone(),
        name: item.name.clone(),
        segment: text::truncate(&item.segment, 120),
        description: item.description.clone(),
        host: item.host.clone(),
        region_text: item.region.clone(),
        batch_size: item.batch_size.to_string(),
        audience_text: text::truncate(&item.audience, 80),
        attempts_text: item.attempts.to_string(),
        contact_email: item.contact_email.clone(),
        timezone_text: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_dashboard_override_views(items: &[StrictDashboardOverride]) -> Vec<StrictDashboardOverrideView> {
    let mut views: Vec<StrictDashboardOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_dashboard_override_view(item));
    }
    views
}

pub fn sum_strict_dashboard_override_view(view: &StrictDashboardOverrideView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_dashboard_override("code-715", "name-73");
        let view = to_strict_dashboard_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_dashboard_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
