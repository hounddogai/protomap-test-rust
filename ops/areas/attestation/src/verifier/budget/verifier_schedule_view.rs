use super::*;

use acme_kit::text;

/// How screens and exports show a verifier schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierScheduleView {
    pub code: String,
    pub name: String,
    pub grace_days_text: String,
    pub host: String,
    pub country: String,
    pub ceiling_text: String,
    pub ttl_seconds: String,
    pub window_days_text: String,
    pub pinned: String,
    pub ends_at_text: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_verifier_schedule_view(item: &VerifierSchedule) -> VerifierScheduleView {
    VerifierScheduleView {
        code: item.code.clone(),
        name: item.name.clone(),
        grace_days_text: item.grace_days.to_string(),
        host: text::truncate(&item.host, 80),
        country: text::truncate(&item.country, 120),
        ceiling_text: format!("{:.2}", item.ceiling),
        ttl_seconds: item.ttl_seconds.to_string(),
        window_days_text: item.window_days.to_string(),
        pinned: format!("{}", item.pinned),
        ends_at_text: item.ends_at.to_string(),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_schedule_views(items: &[VerifierSchedule]) -> Vec<VerifierScheduleView> {
    let mut views: Vec<VerifierScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_schedule_view(item));
    }
    views
}

pub fn sum_verifier_schedule_view(view: &VerifierScheduleView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_ceiling + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_schedule("code-858", "name-946");
        let view = to_verifier_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
