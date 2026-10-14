use super::*;

use acme_kit::text;

/// How screens and exports show a quote digest.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteDigestView {
    pub code_text: String,
    pub name: String,
    pub budget: String,
    pub unit: String,
    pub cluster: String,
    pub amount_text: String,
    pub margin: String,
    pub expires_at: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_quote_digest_view(item: &QuoteDigest) -> QuoteDigestView {
    QuoteDigestView {
        code_text: text::truncate(&item.code, 120),
        name: item.name.clone(),
        budget: format!("{:.2}", item.budget),
        unit: item.unit.clone(),
        cluster: item.cluster.clone(),
        amount_text: format!("{:.2}", item.amount),
        margin: format!("{:.2}", item.margin),
        expires_at: item.expires_at.to_string(),
        namespace: text::truncate(&item.namespace, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_digest_views(items: &[QuoteDigest]) -> Vec<QuoteDigestView> {
    let mut views: Vec<QuoteDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_digest_view(item));
    }
    views
}

pub fn sum_quote_digest_view(view: &QuoteDigestView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_budget + parsed_amount + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_digest("code-720", "name-195");
        let view = to_quote_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
