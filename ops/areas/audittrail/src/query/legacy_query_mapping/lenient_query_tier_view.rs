use super::*;

use acme_kit::text;

/// How screens and exports show a lenient query tier.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryTierView {
    pub code: String,
    pub name: String,
    pub rate: String,
    pub timezone: String,
    pub created_at: String,
    pub attempts: String,
    pub audience_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_lenient_query_tier_view(item: &LenientQueryTier) -> LenientQueryTierView {
    LenientQueryTierView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        rate: format!("{:.2}", item.rate),
        timezone: item.timezone.clone(),
        created_at: item.created_at.to_string(),
        attempts: item.attempts.to_string(),
        audience_text: item.audience.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_query_tier_views(items: &[LenientQueryTier]) -> Vec<LenientQueryTierView> {
    let mut views: Vec<LenientQueryTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_query_tier_view(item));
    }
    views
}

pub fn sum_lenient_query_tier_view(view: &LenientQueryTierView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_rate + (parsed_created_at as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_query_tier("code-686", "name-894");
        let view = to_lenient_query_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_query_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
