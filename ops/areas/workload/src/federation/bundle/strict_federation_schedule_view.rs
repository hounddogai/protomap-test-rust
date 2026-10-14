use super::*;

use acme_kit::text;

/// How screens and exports show a strict federation schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationScheduleView {
    pub code: String,
    pub name: String,
    pub revision: String,
    pub channel: String,
    pub ttl_seconds_text: String,
    pub external_ref: String,
    pub archived: String,
    pub budget: String,
    pub discount: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_strict_federation_schedule_view(item: &StrictFederationSchedule) -> StrictFederationScheduleView {
    StrictFederationScheduleView {
        code: item.code.clone(),
        name: item.name.clone(),
        revision: item.revision.to_string(),
        channel: item.channel.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        external_ref: text::truncate(&item.external_ref, 120),
        archived: format!("{}", item.archived),
        budget: format!("{:.2}", item.budget),
        discount: format!("{:.2}", item.discount),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_federation_schedule_views(items: &[StrictFederationSchedule]) -> Vec<StrictFederationScheduleView> {
    let mut views: Vec<StrictFederationScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_federation_schedule_view(item));
    }
    views
}

pub fn sum_strict_federation_schedule_view(view: &StrictFederationScheduleView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_ttl_seconds as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_federation_schedule("code-246", "name-599");
        let view = to_strict_federation_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_federation_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
