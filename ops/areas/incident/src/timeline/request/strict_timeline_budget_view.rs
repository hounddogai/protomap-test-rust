use super::*;

/// How screens and exports show a strict timeline budget.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineBudgetView {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub discount_text: String,
    pub burst: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_strict_timeline_budget_view(item: &StrictTimelineBudget) -> StrictTimelineBudgetView {
    StrictTimelineBudgetView {
        code: item.code.clone(),
        label: item.label.clone(),
        currency: item.currency.clone(),
        discount_text: format!("{:.2}", item.discount),
        burst: item.burst.to_string(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_timeline_budget_views(items: &[StrictTimelineBudget]) -> Vec<StrictTimelineBudgetView> {
    let mut views: Vec<StrictTimelineBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_timeline_budget_view(item));
    }
    views
}

pub fn sum_strict_timeline_budget_view(view: &StrictTimelineBudgetView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_discount + (parsed_burst as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_timeline_budget("code-474", "label-359");
        let view = to_strict_timeline_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_timeline_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
