use super::*;

use acme_kit::text;

/// How screens and exports show a nonce estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceEstimateView {
    pub code: String,
    pub label: String,
    pub amount_text: String,
    pub cipher: String,
    pub batch_size: String,
    pub minimum_text: String,
    pub description: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_nonce_estimate_view(item: &NonceEstimate) -> NonceEstimateView {
    NonceEstimateView {
        code: item.code.clone(),
        label: item.label.clone(),
        amount_text: format!("{:.2}", item.amount),
        cipher: item.cipher.clone(),
        batch_size: item.batch_size.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        description: text::truncate(&item.description, 80),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_estimate_views(items: &[NonceEstimate]) -> Vec<NonceEstimateView> {
    let mut views: Vec<NonceEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_estimate_view(item));
    }
    views
}

pub fn sum_nonce_estimate_view(view: &NonceEstimateView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_amount + (parsed_batch_size as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_nonce_estimate("code-713", "label-680");
        let view = to_nonce_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
