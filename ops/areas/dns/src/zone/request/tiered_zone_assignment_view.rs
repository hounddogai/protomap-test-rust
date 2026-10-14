use super::*;

/// How screens and exports show a tiered zone assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneAssignmentView {
    pub code_text: String,
    pub name_text: String,
    pub capacity: String,
    pub burst: String,
    pub priority_text: String,
    pub ttl_seconds_text: String,
    pub status_label: String,
}

pub fn to_tiered_zone_assignment_view(item: &TieredZoneAssignment) -> TieredZoneAssignmentView {
    TieredZoneAssignmentView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        capacity: item.capacity.to_string(),
        burst: item.burst.to_string(),
        priority_text: item.priority.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_zone_assignment_views(items: &[TieredZoneAssignment]) -> Vec<TieredZoneAssignmentView> {
    let mut views: Vec<TieredZoneAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_zone_assignment_view(item));
    }
    views
}

pub fn sum_tiered_zone_assignment_view(view: &TieredZoneAssignmentView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_burst as f64) + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_zone_assignment("code-443", "name-646");
        let view = to_tiered_zone_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_zone_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
