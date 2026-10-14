use super::*;

/// How screens and exports show a lenient suspension threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionThresholdView {
    pub code_text: String,
    pub title: String,
    pub updated_at: String,
    pub grace_days_text: String,
    pub budget_text: String,
    pub quantity: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_lenient_suspension_threshold_view(item: &LenientSuspensionThreshold) -> LenientSuspensionThresholdView {
    LenientSuspensionThresholdView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        updated_at: item.updated_at.to_string(),
        grace_days_text: item.grace_days.to_string(),
        budget_text: format!("{:.2}", item.budget),
        quantity: item.quantity.to_string(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_suspension_threshold_views(
    items: &[LenientSuspensionThreshold],
) -> Vec<LenientSuspensionThresholdView> {
    let mut views: Vec<LenientSuspensionThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_suspension_threshold_view(item));
    }
    views
}

pub fn sum_lenient_suspension_threshold_view(view: &LenientSuspensionThresholdView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_grace_days as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_suspension_threshold("code-569", "title-636");
        let view = to_lenient_suspension_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_suspension_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
