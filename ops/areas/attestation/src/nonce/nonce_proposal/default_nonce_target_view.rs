use super::*;

use acme_kit::text;

/// How screens and exports show a default nonce target.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultNonceTargetView {
    pub code: String,
    pub label: String,
    pub visible_text: String,
    pub attempts: String,
    pub floor_value: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_default_nonce_target_view(item: &DefaultNonceTarget) -> DefaultNonceTargetView {
    DefaultNonceTargetView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        visible_text: format!("{}", item.visible),
        attempts: item.attempts.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_nonce_target_views(items: &[DefaultNonceTarget]) -> Vec<DefaultNonceTargetView> {
    let mut views: Vec<DefaultNonceTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_nonce_target_view(item));
    }
    views
}

pub fn sum_default_nonce_target_view(view: &DefaultNonceTargetView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_nonce_target("code-837", "label-345");
        let view = to_default_nonce_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_nonce_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
