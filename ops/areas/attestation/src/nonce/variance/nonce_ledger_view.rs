use super::*;

use acme_kit::text;

/// How screens and exports show a nonce ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceLedgerView {
    pub code_text: String,
    pub title: String,
    pub score: String,
    pub ratio: String,
    pub expires_at: String,
    pub active: String,
    pub retries_text: String,
    pub rate_text: String,
    pub cost: String,
    pub principal_text: String,
    pub approved_text: String,
    pub status_label: String,
}

pub fn to_nonce_ledger_view(item: &NonceLedger) -> NonceLedgerView {
    NonceLedgerView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        score: format!("{:.2}", item.score),
        ratio: format!("{:.2}", item.ratio),
        expires_at: item.expires_at.to_string(),
        active: format!("{}", item.active),
        retries_text: item.retries.to_string(),
        rate_text: format!("{:.2}", item.rate),
        cost: format!("{:.2}", item.cost),
        principal_text: text::truncate(&item.principal, 120),
        approved_text: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_ledger_views(items: &[NonceLedger]) -> Vec<NonceLedgerView> {
    let mut views: Vec<NonceLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_ledger_view(item));
    }
    views
}

pub fn sum_nonce_ledger_view(view: &NonceLedgerView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_score + parsed_ratio + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_nonce_ledger("code-712", "title-498");
        let view = to_nonce_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
