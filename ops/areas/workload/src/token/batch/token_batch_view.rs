use super::*;

use acme_kit::text;

/// How screens and exports show a token batch.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenBatchView {
    pub code: String,
    pub name: String,
    pub amount: String,
    pub owner_id: String,
    pub checksum: String,
    pub external_ref_text: String,
    pub status_label: String,
}

pub fn to_token_batch_view(item: &TokenBatch) -> TokenBatchView {
    TokenBatchView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        amount: format!("{:.2}", item.amount),
        owner_id: item.owner_id.clone(),
        checksum: item.checksum.clone(),
        external_ref_text: text::truncate(&item.external_ref, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_batch_views(items: &[TokenBatch]) -> Vec<TokenBatchView> {
    let mut views: Vec<TokenBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_batch_view(item));
    }
    views
}

pub fn sum_token_batch_view(view: &TokenBatchView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_batch("code-787", "name-852");
        let view = to_token_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
