use super::*;

/// How screens and exports show a allowlist baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBaselineView {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub ceiling: String,
    pub amount_text: String,
    pub rotation_days_text: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_allowlist_baseline_view(item: &AllowlistBaseline) -> AllowlistBaselineView {
    AllowlistBaselineView {
        code: item.code.clone(),
        label: item.label.clone(),
        locale: item.locale.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        amount_text: format!("{:.2}", item.amount),
        rotation_days_text: item.rotation_days.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_baseline_views(items: &[AllowlistBaseline]) -> Vec<AllowlistBaselineView> {
    let mut views: Vec<AllowlistBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_baseline_view(item));
    }
    views
}

pub fn sum_allowlist_baseline_view(view: &AllowlistBaselineView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_ceiling + parsed_amount + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_allowlist_baseline("code-45", "label-412");
        let view = to_allowlist_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_allowlist_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
