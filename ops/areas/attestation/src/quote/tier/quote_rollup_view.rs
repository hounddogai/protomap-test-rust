use super::*;

use acme_kit::text;

/// How screens and exports show a quote rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRollupView {
    pub code: String,
    pub label: String,
    pub burst_text: String,
    pub approved_text: String,
    pub discount: String,
    pub sequence: String,
    pub created_at: String,
    pub currency: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_quote_rollup_view(item: &QuoteRollup) -> QuoteRollupView {
    QuoteRollupView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        burst_text: item.burst.to_string(),
        approved_text: format!("{}", item.approved),
        discount: format!("{:.2}", item.discount),
        sequence: item.sequence.to_string(),
        created_at: item.created_at.to_string(),
        currency: item.currency.clone(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_rollup_views(items: &[QuoteRollup]) -> Vec<QuoteRollupView> {
    let mut views: Vec<QuoteRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_rollup_view(item));
    }
    views
}

pub fn sum_quote_rollup_view(view: &QuoteRollupView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_burst as f64 + parsed_discount + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_rollup("code-657", "label-898");
        let view = to_quote_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
