use super::*;

use acme_kit::text;

/// How screens and exports show a legacy quote batch.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteBatchView {
    pub code: String,
    pub name: String,
    pub retries_text: String,
    pub locale: String,
    pub batch_size: String,
    pub scope_text: String,
    pub key_id_text: String,
    pub minimum: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_legacy_quote_batch_view(item: &LegacyQuoteBatch) -> LegacyQuoteBatchView {
    LegacyQuoteBatchView {
        code: item.code.clone(),
        name: item.name.clone(),
        retries_text: item.retries.to_string(),
        locale: item.locale.clone(),
        batch_size: item.batch_size.to_string(),
        scope_text: item.scope.clone(),
        key_id_text: text::truncate(&item.key_id, 80),
        minimum: format!("{:.2}", item.minimum),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_quote_batch_views(items: &[LegacyQuoteBatch]) -> Vec<LegacyQuoteBatchView> {
    let mut views: Vec<LegacyQuoteBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_quote_batch_view(item));
    }
    views
}

pub fn sum_legacy_quote_batch_view(view: &LegacyQuoteBatchView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_batch_size as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_quote_batch("code-913", "name-259");
        let view = to_legacy_quote_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_quote_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
