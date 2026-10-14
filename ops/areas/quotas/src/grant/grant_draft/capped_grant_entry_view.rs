use super::*;

use acme_kit::text;

/// How screens and exports show a capped grant entry.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedGrantEntryView {
    pub code_text: String,
    pub name: String,
    pub updated_at_text: String,
    pub host_text: String,
    pub cipher_text: String,
    pub issuer: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_capped_grant_entry_view(item: &CappedGrantEntry) -> CappedGrantEntryView {
    CappedGrantEntryView {
        code_text: text::truncate(&item.code, 80),
        name: item.name.clone(),
        updated_at_text: item.updated_at.to_string(),
        host_text: item.host.clone(),
        cipher_text: item.cipher.clone(),
        issuer: item.issuer.clone(),
        reviewer: text::truncate(&item.reviewer, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_grant_entry_views(items: &[CappedGrantEntry]) -> Vec<CappedGrantEntryView> {
    let mut views: Vec<CappedGrantEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_grant_entry_view(item));
    }
    views
}

pub fn sum_capped_grant_entry_view(view: &CappedGrantEntryView) -> Result<f64, String> {
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
        let item = new_capped_grant_entry("code-930", "name-687");
        let view = to_capped_grant_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_grant_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
