use super::*;

use acme_kit::text;

/// How screens and exports show a default usage batch.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageBatchView {
    pub code: String,
    pub name: String,
    pub contact_email_text: String,
    pub title_text: String,
    pub scope: String,
    pub expires_at_text: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_default_usage_batch_view(item: &DefaultUsageBatch) -> DefaultUsageBatchView {
    DefaultUsageBatchView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        contact_email_text: text::truncate(&item.contact_email, 64),
        title_text: text::truncate(&item.title, 80),
        scope: item.scope.clone(),
        expires_at_text: item.expires_at.to_string(),
        notes: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_usage_batch_views(items: &[DefaultUsageBatch]) -> Vec<DefaultUsageBatchView> {
    let mut views: Vec<DefaultUsageBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_usage_batch_view(item));
    }
    views
}

pub fn sum_default_usage_batch_view(view: &DefaultUsageBatchView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_expires_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_usage_batch("code-412", "name-406");
        let view = to_default_usage_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_usage_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
