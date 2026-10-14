use super::*;

use acme_kit::text;

/// How screens and exports show a tiered rule summary.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleSummaryView {
    pub code_text: String,
    pub title: String,
    pub label_text: String,
    pub channel: String,
    pub ends_at: String,
    pub currency: String,
    pub updated_at: String,
    pub cost_text: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_tiered_rule_summary_view(item: &TieredRuleSummary) -> TieredRuleSummaryView {
    TieredRuleSummaryView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 40),
        label_text: item.label.clone(),
        channel: text::truncate(&item.channel, 120),
        ends_at: item.ends_at.to_string(),
        currency: item.currency.clone(),
        updated_at: item.updated_at.to_string(),
        cost_text: format!("{:.2}", item.cost),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_rule_summary_views(items: &[TieredRuleSummary]) -> Vec<TieredRuleSummaryView> {
    let mut views: Vec<TieredRuleSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_rule_summary_view(item));
    }
    views
}

pub fn sum_tiered_rule_summary_view(view: &TieredRuleSummaryView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_updated_at as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_rule_summary("code-346", "title-11");
        let view = to_tiered_rule_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_rule_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
