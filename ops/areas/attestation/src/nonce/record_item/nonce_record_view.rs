use super::*;

use acme_kit::text;

/// How screens and exports show a nonce record.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceRecordView {
    pub code_text: String,
    pub name: String,
    pub country: String,
    pub source: String,
    pub phone: String,
    pub principal: String,
    pub batch_size_text: String,
    pub description_text: String,
    pub owner_id_text: String,
    pub status_label: String,
}

pub fn to_nonce_record_view(item: &NonceRecord) -> NonceRecordView {
    NonceRecordView {
        code_text: text::truncate(&item.code, 120),
        name: item.name.clone(),
        country: item.country.clone(),
        source: text::truncate(&item.source, 64),
        phone: item.phone.clone(),
        principal: text::truncate(&item.principal, 120),
        batch_size_text: item.batch_size.to_string(),
        description_text: item.description.clone(),
        owner_id_text: text::truncate(&item.owner_id, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_record_views(items: &[NonceRecord]) -> Vec<NonceRecordView> {
    let mut views: Vec<NonceRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_record_view(item));
    }
    views
}

pub fn sum_nonce_record_view(view: &NonceRecordView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_nonce_record("code-454", "name-983");
        let view = to_nonce_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
