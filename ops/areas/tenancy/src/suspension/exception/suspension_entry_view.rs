use super::*;

use acme_kit::text;

/// How screens and exports show a suspension entry.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionEntryView {
    pub code: String,
    pub title: String,
    pub category: String,
    pub port: String,
    pub duration_minutes: String,
    pub revision: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_suspension_entry_view(item: &SuspensionEntry) -> SuspensionEntryView {
    SuspensionEntryView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        category: item.category.clone(),
        port: item.port.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        revision: item.revision.to_string(),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_entry_views(items: &[SuspensionEntry]) -> Vec<SuspensionEntryView> {
    let mut views: Vec<SuspensionEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_entry_view(item));
    }
    views
}

pub fn sum_suspension_entry_view(view: &SuspensionEntryView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_port as f64 + (parsed_duration_minutes as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_entry("code-800", "title-513");
        let view = to_suspension_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
