use super::*;

use acme_kit::text;

/// How screens and exports show a canary entry.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryEntryView {
    pub code_text: String,
    pub label: String,
    pub channel_text: String,
    pub reviewer_text: String,
    pub source: String,
    pub window_days: String,
    pub locale: String,
    pub phone_text: String,
    pub status_label: String,
}

pub fn to_canary_entry_view(item: &CanaryEntry) -> CanaryEntryView {
    CanaryEntryView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        channel_text: item.channel.clone(),
        reviewer_text: item.reviewer.clone(),
        source: text::truncate(&item.source, 64),
        window_days: item.window_days.to_string(),
        locale: item.locale.clone(),
        phone_text: text::truncate(&item.phone, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_entry_views(items: &[CanaryEntry]) -> Vec<CanaryEntryView> {
    let mut views: Vec<CanaryEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_entry_view(item));
    }
    views
}

pub fn sum_canary_entry_view(view: &CanaryEntryView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_window_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_entry("code-797", "label-800");
        let view = to_canary_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
