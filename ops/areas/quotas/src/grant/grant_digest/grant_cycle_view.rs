use super::*;

use acme_kit::text;

/// How screens and exports show a grant cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantCycleView {
    pub code_text: String,
    pub name: String,
    pub quantity: String,
    pub created_at: String,
    pub active: String,
    pub external_ref_text: String,
    pub updated_at: String,
    pub window_days_text: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_grant_cycle_view(item: &GrantCycle) -> GrantCycleView {
    GrantCycleView {
        code_text: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 40),
        quantity: item.quantity.to_string(),
        created_at: item.created_at.to_string(),
        active: format!("{}", item.active),
        external_ref_text: item.external_ref.clone(),
        updated_at: item.updated_at.to_string(),
        window_days_text: item.window_days.to_string(),
        reviewer: text::truncate(&item.reviewer, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_cycle_views(items: &[GrantCycle]) -> Vec<GrantCycleView> {
    let mut views: Vec<GrantCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_cycle_view(item));
    }
    views
}

pub fn sum_grant_cycle_view(view: &GrantCycleView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_created_at as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_cycle("code-802", "name-903");
        let view = to_grant_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
