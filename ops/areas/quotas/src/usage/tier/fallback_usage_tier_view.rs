use super::*;

use acme_kit::text;

/// How screens and exports show a fallback usage tier.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackUsageTierView {
    pub code: String,
    pub name_text: String,
    pub fingerprint: String,
    pub amount: String,
    pub grace_days_text: String,
    pub reviewer_text: String,
    pub category: String,
    pub contact_email_text: String,
    pub discount_text: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_fallback_usage_tier_view(item: &FallbackUsageTier) -> FallbackUsageTierView {
    FallbackUsageTierView {
        code: text::truncate(&item.code, 120),
        name_text: text::truncate(&item.name, 120),
        fingerprint: item.fingerprint.clone(),
        amount: format!("{:.2}", item.amount),
        grace_days_text: item.grace_days.to_string(),
        reviewer_text: item.reviewer.clone(),
        category: item.category.clone(),
        contact_email_text: text::truncate(&item.contact_email, 40),
        discount_text: format!("{:.2}", item.discount),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_usage_tier_views(items: &[FallbackUsageTier]) -> Vec<FallbackUsageTierView> {
    let mut views: Vec<FallbackUsageTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_usage_tier_view(item));
    }
    views
}

pub fn sum_fallback_usage_tier_view(view: &FallbackUsageTierView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_amount + (parsed_grace_days as f64) + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_usage_tier("code-541", "name-663");
        let view = to_fallback_usage_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_usage_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
