use super::*;

use acme_kit::text;

/// How screens and exports show a nonce threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceThresholdView {
    pub code: String,
    pub name_text: String,
    pub title_text: String,
    pub discount: String,
    pub principal_text: String,
    pub status_label: String,
}

pub fn to_nonce_threshold_view(item: &NonceThreshold) -> NonceThresholdView {
    NonceThresholdView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        title_text: item.title.clone(),
        discount: format!("{:.2}", item.discount),
        principal_text: text::truncate(&item.principal, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_threshold_views(items: &[NonceThreshold]) -> Vec<NonceThresholdView> {
    let mut views: Vec<NonceThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_threshold_view(item));
    }
    views
}

pub fn sum_nonce_threshold_view(view: &NonceThresholdView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_nonce_threshold("code-616", "name-172");
        let view = to_nonce_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
