use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemCycleView {
    pub code: String,
    pub label: String,
    pub maximum: String,
    pub window_days: String,
    pub cost: String,
    pub minimum: String,
    pub quantity: String,
    pub capacity_text: String,
    pub status_label: String,
}

pub fn to_postmortem_cycle_view(item: &PostmortemCycle) -> PostmortemCycleView {
    PostmortemCycleView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        maximum: format!("{:.2}", item.maximum),
        window_days: item.window_days.to_string(),
        cost: format!("{:.2}", item.cost),
        minimum: format!("{:.2}", item.minimum),
        quantity: item.quantity.to_string(),
        capacity_text: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_cycle_views(items: &[PostmortemCycle]) -> Vec<PostmortemCycleView> {
    let mut views: Vec<PostmortemCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_cycle_view(item));
    }
    views
}

pub fn sum_postmortem_cycle_view(view: &PostmortemCycleView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_maximum + (parsed_window_days as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_postmortem_cycle("code-973", "label-768");
        let view = to_postmortem_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_postmortem_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
