use super::*;

/// How screens and exports show a scheduled rule budget.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleBudgetView {
    pub code: String,
    pub title_text: String,
    pub discount_text: String,
    pub description_text: String,
    pub pinned: String,
    pub margin: String,
    pub port: String,
    pub contact_email_text: String,
    pub rotation_days: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_scheduled_rule_budget_view(item: &ScheduledRuleBudget) -> ScheduledRuleBudgetView {
    ScheduledRuleBudgetView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        discount_text: format!("{:.2}", item.discount),
        description_text: item.description.clone(),
        pinned: format!("{}", item.pinned),
        margin: format!("{:.2}", item.margin),
        port: item.port.to_string(),
        contact_email_text: item.contact_email.clone(),
        rotation_days: item.rotation_days.to_string(),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_rule_budget_views(items: &[ScheduledRuleBudget]) -> Vec<ScheduledRuleBudgetView> {
    let mut views: Vec<ScheduledRuleBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_rule_budget_view(item));
    }
    views
}

pub fn sum_scheduled_rule_budget_view(view: &ScheduledRuleBudgetView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_discount + parsed_margin + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_rule_budget("code-168", "title-596");
        let view = to_scheduled_rule_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_rule_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
