use super::*;

use acme_kit::text;

/// How screens and exports show a batched nonce template.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedNonceTemplateView {
    pub code: String,
    pub title: String,
    pub quantity: String,
    pub fingerprint: String,
    pub source_text: String,
    pub label: String,
    pub name: String,
    pub attempts_text: String,
    pub status_label: String,
}

pub fn to_batched_nonce_template_view(item: &BatchedNonceTemplate) -> BatchedNonceTemplateView {
    BatchedNonceTemplateView {
        code: item.code.clone(),
        title: item.title.clone(),
        quantity: item.quantity.to_string(),
        fingerprint: item.fingerprint.clone(),
        source_text: text::truncate(&item.source, 64),
        label: item.label.clone(),
        name: item.name.clone(),
        attempts_text: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_nonce_template_views(items: &[BatchedNonceTemplate]) -> Vec<BatchedNonceTemplateView> {
    let mut views: Vec<BatchedNonceTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_nonce_template_view(item));
    }
    views
}

pub fn sum_batched_nonce_template_view(view: &BatchedNonceTemplateView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_nonce_template("code-325", "title-810");
        let view = to_batched_nonce_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_nonce_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
