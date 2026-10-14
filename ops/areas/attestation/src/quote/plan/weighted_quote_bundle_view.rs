use super::*;

use acme_kit::text;

/// How screens and exports show a weighted quote bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQuoteBundleView {
    pub code: String,
    pub label: String,
    pub discount_text: String,
    pub expires_at: String,
    pub archived_text: String,
    pub source_text: String,
    pub budget: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_weighted_quote_bundle_view(item: &WeightedQuoteBundle) -> WeightedQuoteBundleView {
    WeightedQuoteBundleView {
        code: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 64),
        discount_text: format!("{:.2}", item.discount),
        expires_at: item.expires_at.to_string(),
        archived_text: format!("{}", item.archived),
        source_text: text::truncate(&item.source, 40),
        budget: format!("{:.2}", item.budget),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_quote_bundle_views(items: &[WeightedQuoteBundle]) -> Vec<WeightedQuoteBundleView> {
    let mut views: Vec<WeightedQuoteBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_quote_bundle_view(item));
    }
    views
}

pub fn sum_weighted_quote_bundle_view(view: &WeightedQuoteBundleView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_discount + (parsed_expires_at as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_quote_bundle("code-708", "label-449");
        let view = to_weighted_quote_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_quote_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
