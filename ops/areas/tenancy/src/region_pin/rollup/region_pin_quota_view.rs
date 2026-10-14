use super::*;

use acme_kit::text;

/// How screens and exports show a region pin quota.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinQuotaView {
    pub code_text: String,
    pub name: String,
    pub capacity: String,
    pub host_text: String,
    pub priority: String,
    pub budget: String,
    pub active_text: String,
    pub reviewer: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_region_pin_quota_view(item: &RegionPinQuota) -> RegionPinQuotaView {
    RegionPinQuotaView {
        code_text: text::truncate(&item.code, 40),
        name: item.name.clone(),
        capacity: item.capacity.to_string(),
        host_text: item.host.clone(),
        priority: item.priority.to_string(),
        budget: format!("{:.2}", item.budget),
        active_text: format!("{}", item.active),
        reviewer: item.reviewer.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_quota_views(items: &[RegionPinQuota]) -> Vec<RegionPinQuotaView> {
    let mut views: Vec<RegionPinQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_quota_view(item));
    }
    views
}

pub fn sum_region_pin_quota_view(view: &RegionPinQuotaView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_priority as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_quota("code-654", "name-688");
        let view = to_region_pin_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
