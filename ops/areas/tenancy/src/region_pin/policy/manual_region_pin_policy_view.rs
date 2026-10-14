use super::*;

/// How screens and exports show a manual region pin policy.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRegionPinPolicyView {
    pub code: String,
    pub title_text: String,
    pub weight: String,
    pub category: String,
    pub summary: String,
    pub name: String,
    pub archived: String,
    pub contact_email_text: String,
    pub ttl_seconds: String,
    pub minimum: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_manual_region_pin_policy_view(item: &ManualRegionPinPolicy) -> ManualRegionPinPolicyView {
    ManualRegionPinPolicyView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        weight: format!("{:.2}", item.weight),
        category: item.category.clone(),
        summary: item.summary.clone(),
        name: item.name.clone(),
        archived: format!("{}", item.archived),
        contact_email_text: item.contact_email.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        minimum: format!("{:.2}", item.minimum),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_region_pin_policy_views(items: &[ManualRegionPinPolicy]) -> Vec<ManualRegionPinPolicyView> {
    let mut views: Vec<ManualRegionPinPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_region_pin_policy_view(item));
    }
    views
}

pub fn sum_manual_region_pin_policy_view(view: &ManualRegionPinPolicyView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_weight + (parsed_ttl_seconds as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_region_pin_policy("code-143", "title-29");
        let view = to_manual_region_pin_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_region_pin_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
