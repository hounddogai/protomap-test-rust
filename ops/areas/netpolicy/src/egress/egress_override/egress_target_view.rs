use super::*;

use acme_kit::text;

/// How screens and exports show a egress target.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressTargetView {
    pub code: String,
    pub name: String,
    pub approved: String,
    pub audience: String,
    pub priority: String,
    pub unit: String,
    pub visible: String,
    pub owner_id: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_egress_target_view(item: &EgressTarget) -> EgressTargetView {
    EgressTargetView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        approved: format!("{}", item.approved),
        audience: item.audience.clone(),
        priority: item.priority.to_string(),
        unit: item.unit.clone(),
        visible: format!("{}", item.visible),
        owner_id: text::truncate(&item.owner_id, 64),
        summary: text::truncate(&item.summary, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_target_views(items: &[EgressTarget]) -> Vec<EgressTargetView> {
    let mut views: Vec<EgressTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_target_view(item));
    }
    views
}

pub fn sum_egress_target_view(view: &EgressTargetView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_target("code-172", "name-382");
        let view = to_egress_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
