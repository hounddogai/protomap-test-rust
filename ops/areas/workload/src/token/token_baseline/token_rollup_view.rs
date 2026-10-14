use super::*;

use acme_kit::text;

/// How screens and exports show a token rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenRollupView {
    pub code: String,
    pub label_text: String,
    pub approved_text: String,
    pub notes: String,
    pub expires_at: String,
    pub unit: String,
    pub locale: String,
    pub capacity: String,
    pub checksum_text: String,
    pub summary_text: String,
    pub external_ref_text: String,
    pub status_label: String,
}

pub fn to_token_rollup_view(item: &TokenRollup) -> TokenRollupView {
    TokenRollupView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        approved_text: format!("{}", item.approved),
        notes: text::truncate(&item.notes, 40),
        expires_at: item.expires_at.to_string(),
        unit: text::truncate(&item.unit, 64),
        locale: item.locale.clone(),
        capacity: item.capacity.to_string(),
        checksum_text: item.checksum.clone(),
        summary_text: item.summary.clone(),
        external_ref_text: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_rollup_views(items: &[TokenRollup]) -> Vec<TokenRollupView> {
    let mut views: Vec<TokenRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_rollup_view(item));
    }
    views
}

pub fn sum_token_rollup_view(view: &TokenRollupView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_rollup("code-981", "label-788");
        let view = to_token_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
