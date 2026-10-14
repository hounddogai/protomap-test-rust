use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled canary policy.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanaryPolicyView {
    pub code_text: String,
    pub name: String,
    pub audience_text: String,
    pub notes: String,
    pub unit: String,
    pub floor_value: String,
    pub category: String,
    pub checksum: String,
    pub phone_text: String,
    pub status_label: String,
}

pub fn to_scheduled_canary_policy_view(item: &ScheduledCanaryPolicy) -> ScheduledCanaryPolicyView {
    ScheduledCanaryPolicyView {
        code_text: text::truncate(&item.code, 120),
        name: item.name.clone(),
        audience_text: item.audience.clone(),
        notes: item.notes.clone(),
        unit: text::truncate(&item.unit, 40),
        floor_value: format!("{:.2}", item.floor_value),
        category: item.category.clone(),
        checksum: item.checksum.clone(),
        phone_text: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_canary_policy_views(items: &[ScheduledCanaryPolicy]) -> Vec<ScheduledCanaryPolicyView> {
    let mut views: Vec<ScheduledCanaryPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_canary_policy_view(item));
    }
    views
}

pub fn sum_scheduled_canary_policy_view(view: &ScheduledCanaryPolicyView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_canary_policy("code-750", "name-707");
        let view = to_scheduled_canary_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_canary_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
