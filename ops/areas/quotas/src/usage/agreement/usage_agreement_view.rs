use super::*;

use acme_kit::text;

/// How screens and exports show a usage agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageAgreementView {
    pub code: String,
    pub name: String,
    pub window_days_text: String,
    pub capacity: String,
    pub principal_text: String,
    pub minimum_text: String,
    pub expires_at: String,
    pub source: String,
    pub category_text: String,
    pub status_label: String,
}

pub fn to_usage_agreement_view(item: &UsageAgreement) -> UsageAgreementView {
    UsageAgreementView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        window_days_text: item.window_days.to_string(),
        capacity: item.capacity.to_string(),
        principal_text: text::truncate(&item.principal, 120),
        minimum_text: format!("{:.2}", item.minimum),
        expires_at: item.expires_at.to_string(),
        source: text::truncate(&item.source, 64),
        category_text: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_agreement_views(items: &[UsageAgreement]) -> Vec<UsageAgreementView> {
    let mut views: Vec<UsageAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_agreement_view(item));
    }
    views
}

pub fn sum_usage_agreement_view(view: &UsageAgreementView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_capacity as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_agreement("code-168", "name-679");
        let view = to_usage_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
