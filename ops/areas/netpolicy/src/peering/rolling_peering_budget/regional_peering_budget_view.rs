use super::*;

use acme_kit::text;

/// How screens and exports show a regional peering budget.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringBudgetView {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub minimum: String,
    pub version_text: String,
    pub starts_at: String,
    pub contact_email: String,
    pub cipher: String,
    pub segment: String,
    pub attempts_text: String,
    pub status_label: String,
}

pub fn to_regional_peering_budget_view(item: &RegionalPeeringBudget) -> RegionalPeeringBudgetView {
    RegionalPeeringBudgetView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        audience: item.audience.clone(),
        minimum: format!("{:.2}", item.minimum),
        version_text: item.version.to_string(),
        starts_at: item.starts_at.to_string(),
        contact_email: text::truncate(&item.contact_email, 120),
        cipher: text::truncate(&item.cipher, 64),
        segment: item.segment.clone(),
        attempts_text: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_peering_budget_views(items: &[RegionalPeeringBudget]) -> Vec<RegionalPeeringBudgetView> {
    let mut views: Vec<RegionalPeeringBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_peering_budget_view(item));
    }
    views
}

pub fn sum_regional_peering_budget_view(view: &RegionalPeeringBudgetView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_minimum + (parsed_version as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_peering_budget("code-492", "title-294");
        let view = to_regional_peering_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_peering_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
