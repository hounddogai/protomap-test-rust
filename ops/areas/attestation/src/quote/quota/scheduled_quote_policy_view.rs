use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled quote policy.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuotePolicyView {
    pub code_text: String,
    pub name: String,
    pub weight: String,
    pub ratio: String,
    pub issuer_text: String,
    pub status_label: String,
}

pub fn to_scheduled_quote_policy_view(item: &ScheduledQuotePolicy) -> ScheduledQuotePolicyView {
    ScheduledQuotePolicyView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        weight: format!("{:.2}", item.weight),
        ratio: format!("{:.2}", item.ratio),
        issuer_text: text::truncate(&item.issuer, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_quote_policy_views(items: &[ScheduledQuotePolicy]) -> Vec<ScheduledQuotePolicyView> {
    let mut views: Vec<ScheduledQuotePolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_quote_policy_view(item));
    }
    views
}

pub fn sum_scheduled_quote_policy_view(view: &ScheduledQuotePolicyView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_weight + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_quote_policy("code-105", "name-400");
        let view = to_scheduled_quote_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_quote_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
