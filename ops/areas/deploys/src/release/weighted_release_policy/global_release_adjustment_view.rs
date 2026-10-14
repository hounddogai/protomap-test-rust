use super::*;

/// How screens and exports show a global release adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAdjustmentView {
    pub code_text: String,
    pub title_text: String,
    pub name: String,
    pub quantity: String,
    pub sequence: String,
    pub namespace_text: String,
    pub quota_units_text: String,
    pub window_days: String,
    pub priority: String,
    pub limit: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_global_release_adjustment_view(item: &GlobalReleaseAdjustment) -> GlobalReleaseAdjustmentView {
    GlobalReleaseAdjustmentView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        name: item.name.clone(),
        quantity: item.quantity.to_string(),
        sequence: item.sequence.to_string(),
        namespace_text: item.namespace.clone(),
        quota_units_text: item.quota_units.to_string(),
        window_days: item.window_days.to_string(),
        priority: item.priority.to_string(),
        limit: item.limit.to_string(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_release_adjustment_views(items: &[GlobalReleaseAdjustment]) -> Vec<GlobalReleaseAdjustmentView> {
    let mut views: Vec<GlobalReleaseAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_release_adjustment_view(item));
    }
    views
}

pub fn sum_global_release_adjustment_view(view: &GlobalReleaseAdjustmentView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_sequence as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_release_adjustment("code-454", "title-319");
        let view = to_global_release_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_release_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
