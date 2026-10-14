use super::*;

use acme_kit::text;

/// How screens and exports show a regional quote budget.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQuoteBudgetView {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub fingerprint: String,
    pub host_text: String,
    pub external_ref_text: String,
    pub ratio_text: String,
    pub key_id_text: String,
    pub visible: String,
    pub rate: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_regional_quote_budget_view(item: &RegionalQuoteBudget) -> RegionalQuoteBudgetView {
    RegionalQuoteBudgetView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        unit: item.unit.clone(),
        fingerprint: item.fingerprint.clone(),
        host_text: item.host.clone(),
        external_ref_text: item.external_ref.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        key_id_text: item.key_id.clone(),
        visible: format!("{}", item.visible),
        rate: format!("{:.2}", item.rate),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_quote_budget_views(items: &[RegionalQuoteBudget]) -> Vec<RegionalQuoteBudgetView> {
    let mut views: Vec<RegionalQuoteBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_quote_budget_view(item));
    }
    views
}

pub fn sum_regional_quote_budget_view(view: &RegionalQuoteBudgetView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_ratio + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_quote_budget("code-248", "name-928");
        let view = to_regional_quote_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_quote_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
