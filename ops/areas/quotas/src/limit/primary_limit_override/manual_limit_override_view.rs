use super::*;

/// How screens and exports show a manual limit override.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitOverrideView {
    pub code: String,
    pub name: String,
    pub ttl_seconds: String,
    pub budget: String,
    pub quota_units: String,
    pub ceiling: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_manual_limit_override_view(item: &ManualLimitOverride) -> ManualLimitOverrideView {
    ManualLimitOverrideView {
        code: item.code.clone(),
        name: item.name.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        budget: format!("{:.2}", item.budget),
        quota_units: item.quota_units.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_limit_override_views(items: &[ManualLimitOverride]) -> Vec<ManualLimitOverrideView> {
    let mut views: Vec<ManualLimitOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_limit_override_view(item));
    }
    views
}

pub fn sum_manual_limit_override_view(view: &ManualLimitOverrideView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_budget + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_limit_override("code-600", "name-335");
        let view = to_manual_limit_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_limit_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
