use super::*;

use acme_kit::text;

/// How screens and exports show a quote record.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRecordView {
    pub code: String,
    pub label: String,
    pub amount: String,
    pub visible_text: String,
    pub segment: String,
    pub key_id: String,
    pub external_ref: String,
    pub budget: String,
    pub fingerprint_text: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_quote_record_view(item: &QuoteRecord) -> QuoteRecordView {
    QuoteRecordView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        amount: format!("{:.2}", item.amount),
        visible_text: format!("{}", item.visible),
        segment: item.segment.clone(),
        key_id: item.key_id.clone(),
        external_ref: item.external_ref.clone(),
        budget: format!("{:.2}", item.budget),
        fingerprint_text: item.fingerprint.clone(),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_record_views(items: &[QuoteRecord]) -> Vec<QuoteRecordView> {
    let mut views: Vec<QuoteRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_record_view(item));
    }
    views
}

pub fn sum_quote_record_view(view: &QuoteRecordView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_amount + parsed_budget + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_record("code-541", "label-225");
        let view = to_quote_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
