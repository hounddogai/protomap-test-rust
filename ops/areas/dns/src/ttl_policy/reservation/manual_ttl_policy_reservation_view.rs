use super::*;

use acme_kit::text;

/// How screens and exports show a manual ttl policy reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyReservationView {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub category: String,
    pub priority: String,
    pub ratio_text: String,
    pub grace_days: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_manual_ttl_policy_reservation_view(item: &ManualTtlPolicyReservation) -> ManualTtlPolicyReservationView {
    ManualTtlPolicyReservationView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        scope: item.scope.clone(),
        category: item.category.clone(),
        priority: item.priority.to_string(),
        ratio_text: format!("{:.2}", item.ratio),
        grace_days: item.grace_days.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_ttl_policy_reservation_views(
    items: &[ManualTtlPolicyReservation],
) -> Vec<ManualTtlPolicyReservationView> {
    let mut views: Vec<ManualTtlPolicyReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_ttl_policy_reservation_view(item));
    }
    views
}

pub fn sum_manual_ttl_policy_reservation_view(view: &ManualTtlPolicyReservationView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_priority as f64 + parsed_ratio + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_ttl_policy_reservation("code-277", "title-101");
        let view = to_manual_ttl_policy_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_ttl_policy_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
