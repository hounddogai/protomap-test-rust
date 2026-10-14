use super::*;

use acme_kit::text;

/// How screens and exports show a alert entry.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertEntryView {
    pub code: String,
    pub title_text: String,
    pub phone: String,
    pub expires_at_text: String,
    pub batch_size_text: String,
    pub cipher: String,
    pub audience: String,
    pub created_at_text: String,
    pub status_label: String,
}

pub fn to_alert_entry_view(item: &AlertEntry) -> AlertEntryView {
    AlertEntryView {
        code: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        phone: item.phone.clone(),
        expires_at_text: item.expires_at.to_string(),
        batch_size_text: item.batch_size.to_string(),
        cipher: item.cipher.clone(),
        audience: item.audience.clone(),
        created_at_text: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_entry_views(items: &[AlertEntry]) -> Vec<AlertEntryView> {
    let mut views: Vec<AlertEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_entry_view(item));
    }
    views
}

pub fn sum_alert_entry_view(view: &AlertEntryView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_batch_size as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_alert_entry("code-220", "title-936");
        let view = to_alert_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_alert_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
