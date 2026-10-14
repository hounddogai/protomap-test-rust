use super::*;

use acme_kit::text;

/// How screens and exports show a rolling limit group.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitGroupView {
    pub code: String,
    pub name: String,
    pub pinned_text: String,
    pub quantity: String,
    pub active: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_rolling_limit_group_view(item: &RollingLimitGroup) -> RollingLimitGroupView {
    RollingLimitGroupView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        pinned_text: format!("{}", item.pinned),
        quantity: item.quantity.to_string(),
        active: format!("{}", item.active),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_limit_group_views(items: &[RollingLimitGroup]) -> Vec<RollingLimitGroupView> {
    let mut views: Vec<RollingLimitGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_limit_group_view(item));
    }
    views
}

pub fn sum_rolling_limit_group_view(view: &RollingLimitGroupView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_limit_group("code-407", "name-261");
        let view = to_rolling_limit_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_limit_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
