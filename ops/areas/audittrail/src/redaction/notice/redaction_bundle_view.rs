use super::*;

use acme_kit::text;

/// How screens and exports show a redaction bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBundleView {
    pub code: String,
    pub title_text: String,
    pub category: String,
    pub created_at: String,
    pub maximum_text: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_redaction_bundle_view(item: &RedactionBundle) -> RedactionBundleView {
    RedactionBundleView {
        code: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        category: item.category.clone(),
        created_at: item.created_at.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_bundle_views(items: &[RedactionBundle]) -> Vec<RedactionBundleView> {
    let mut views: Vec<RedactionBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_bundle_view(item));
    }
    views
}

pub fn sum_redaction_bundle_view(view: &RedactionBundleView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_maximum + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_redaction_bundle("code-350", "title-807");
        let view = to_redaction_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_redaction_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
