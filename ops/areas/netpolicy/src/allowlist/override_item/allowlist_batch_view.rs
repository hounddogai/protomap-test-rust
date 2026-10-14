use super::*;

use acme_kit::text;

/// How screens and exports show a allowlist batch.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBatchView {
    pub code: String,
    pub name: String,
    pub owner_id_text: String,
    pub reviewer: String,
    pub issuer: String,
    pub key_id_text: String,
    pub visible_text: String,
    pub updated_at_text: String,
    pub status_label: String,
}

pub fn to_allowlist_batch_view(item: &AllowlistBatch) -> AllowlistBatchView {
    AllowlistBatchView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        owner_id_text: item.owner_id.clone(),
        reviewer: item.reviewer.clone(),
        issuer: item.issuer.clone(),
        key_id_text: text::truncate(&item.key_id, 80),
        visible_text: format!("{}", item.visible),
        updated_at_text: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_batch_views(items: &[AllowlistBatch]) -> Vec<AllowlistBatchView> {
    let mut views: Vec<AllowlistBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_batch_view(item));
    }
    views
}

pub fn sum_allowlist_batch_view(view: &AllowlistBatchView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_updated_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_allowlist_batch("code-430", "name-730");
        let view = to_allowlist_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_allowlist_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
