use super::*;

use acme_kit::text;

/// How screens and exports show a strict query profile.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryProfileView {
    pub code_text: String,
    pub name_text: String,
    pub reviewer: String,
    pub duration_minutes: String,
    pub grace_days: String,
    pub timezone: String,
    pub retries: String,
    pub owner_id: String,
    pub attempts: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_strict_query_profile_view(item: &StrictQueryProfile) -> StrictQueryProfileView {
    StrictQueryProfileView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        reviewer: text::truncate(&item.reviewer, 80),
        duration_minutes: item.duration_minutes.to_string(),
        grace_days: item.grace_days.to_string(),
        timezone: item.timezone.clone(),
        retries: item.retries.to_string(),
        owner_id: item.owner_id.clone(),
        attempts: item.attempts.to_string(),
        unit: text::truncate(&item.unit, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_query_profile_views(items: &[StrictQueryProfile]) -> Vec<StrictQueryProfileView> {
    let mut views: Vec<StrictQueryProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_query_profile_view(item));
    }
    views
}

pub fn sum_strict_query_profile_view(view: &StrictQueryProfileView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_grace_days as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_query_profile("code-176", "name-859");
        let view = to_strict_query_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_query_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
