use super::*;

use acme_kit::text;

/// How screens and exports show a quote quota.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteQuotaView {
    pub code: String,
    pub label_text: String,
    pub approved: String,
    pub weight_text: String,
    pub namespace: String,
    pub timezone: String,
    pub cipher: String,
    pub expires_at: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_quote_quota_view(item: &QuoteQuota) -> QuoteQuotaView {
    QuoteQuotaView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        approved: format!("{}", item.approved),
        weight_text: format!("{:.2}", item.weight),
        namespace: text::truncate(&item.namespace, 120),
        timezone: item.timezone.clone(),
        cipher: item.cipher.clone(),
        expires_at: item.expires_at.to_string(),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_quota_views(items: &[QuoteQuota]) -> Vec<QuoteQuotaView> {
    let mut views: Vec<QuoteQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_quota_view(item));
    }
    views
}

pub fn sum_quote_quota_view(view: &QuoteQuotaView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_weight + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_quota("code-27", "label-172");
        let view = to_quote_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
