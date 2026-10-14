use super::*;

use acme_kit::text;

/// How screens and exports show a identity entry.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityEntryView {
    pub code_text: String,
    pub title: String,
    pub external_ref_text: String,
    pub channel_text: String,
    pub locale: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_identity_entry_view(item: &IdentityEntry) -> IdentityEntryView {
    IdentityEntryView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        external_ref_text: text::truncate(&item.external_ref, 120),
        channel_text: text::truncate(&item.channel, 80),
        locale: item.locale.clone(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_entry_views(items: &[IdentityEntry]) -> Vec<IdentityEntryView> {
    let mut views: Vec<IdentityEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_entry_view(item));
    }
    views
}

pub fn sum_identity_entry_view(view: &IdentityEntryView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_version as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_entry("code-318", "title-962");
        let view = to_identity_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
