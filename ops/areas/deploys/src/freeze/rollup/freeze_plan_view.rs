use super::*;

use acme_kit::text;

/// How screens and exports show a freeze plan.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezePlanView {
    pub code: String,
    pub name: String,
    pub phone_text: String,
    pub ceiling_text: String,
    pub description_text: String,
    pub cluster: String,
    pub floor_value: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_freeze_plan_view(item: &FreezePlan) -> FreezePlanView {
    FreezePlanView {
        code: item.code.clone(),
        name: item.name.clone(),
        phone_text: text::truncate(&item.phone, 80),
        ceiling_text: format!("{:.2}", item.ceiling),
        description_text: text::truncate(&item.description, 80),
        cluster: item.cluster.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_plan_views(items: &[FreezePlan]) -> Vec<FreezePlanView> {
    let mut views: Vec<FreezePlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_plan_view(item));
    }
    views
}

pub fn sum_freeze_plan_view(view: &FreezePlanView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_ceiling + parsed_floor_value + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_plan("code-952", "name-540");
        let view = to_freeze_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
