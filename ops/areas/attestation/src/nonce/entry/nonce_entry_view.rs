use super::*;

use acme_kit::text;

/// How screens and exports show a nonce entry.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceEntryView {
    pub code: String,
    pub title: String,
    pub key_id_text: String,
    pub label: String,
    pub host_text: String,
    pub region: String,
    pub discount: String,
    pub maximum: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_nonce_entry_view(item: &NonceEntry) -> NonceEntryView {
    NonceEntryView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        key_id_text: item.key_id.clone(),
        label: text::truncate(&item.label, 40),
        host_text: text::truncate(&item.host, 40),
        region: item.region.clone(),
        discount: format!("{:.2}", item.discount),
        maximum: format!("{:.2}", item.maximum),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_entry_views(items: &[NonceEntry]) -> Vec<NonceEntryView> {
    let mut views: Vec<NonceEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_entry_view(item));
    }
    views
}

pub fn sum_nonce_entry_view(view: &NonceEntryView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_discount + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_nonce_entry("code-553", "title-689");
        let view = to_nonce_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
