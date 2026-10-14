use super::*;

use acme_kit::text;

/// How screens and exports show a tiered nonce target.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceTargetView {
    pub code: String,
    pub title: String,
    pub cipher_text: String,
    pub weight: String,
    pub attempts: String,
    pub fingerprint: String,
    pub tenant_id_text: String,
    pub pinned_text: String,
    pub ends_at: String,
    pub name: String,
    pub status_label: String,
}

pub fn to_tiered_nonce_target_view(item: &TieredNonceTarget) -> TieredNonceTargetView {
    TieredNonceTargetView {
        code: item.code.clone(),
        title: item.title.clone(),
        cipher_text: item.cipher.clone(),
        weight: format!("{:.2}", item.weight),
        attempts: item.attempts.to_string(),
        fingerprint: item.fingerprint.clone(),
        tenant_id_text: text::truncate(&item.tenant_id, 40),
        pinned_text: format!("{}", item.pinned),
        ends_at: item.ends_at.to_string(),
        name: text::truncate(&item.name, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_nonce_target_views(items: &[TieredNonceTarget]) -> Vec<TieredNonceTargetView> {
    let mut views: Vec<TieredNonceTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_nonce_target_view(item));
    }
    views
}

pub fn sum_tiered_nonce_target_view(view: &TieredNonceTargetView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_weight + (parsed_attempts as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_nonce_target("code-377", "title-384");
        let view = to_tiered_nonce_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_nonce_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
