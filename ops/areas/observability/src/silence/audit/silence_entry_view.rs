use super::*;

use acme_kit::text;

/// How screens and exports show a silence entry.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceEntryView {
    pub code: String,
    pub label: String,
    pub updated_at: String,
    pub title_text: String,
    pub rate_text: String,
    pub description: String,
    pub discount: String,
    pub source: String,
    pub contact_email: String,
    pub archived_text: String,
    pub reviewer_text: String,
    pub status_label: String,
}

pub fn to_silence_entry_view(item: &SilenceEntry) -> SilenceEntryView {
    SilenceEntryView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        updated_at: item.updated_at.to_string(),
        title_text: text::truncate(&item.title, 80),
        rate_text: format!("{:.2}", item.rate),
        description: item.description.clone(),
        discount: format!("{:.2}", item.discount),
        source: text::truncate(&item.source, 40),
        contact_email: text::truncate(&item.contact_email, 64),
        archived_text: format!("{}", item.archived),
        reviewer_text: text::truncate(&item.reviewer, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_entry_views(items: &[SilenceEntry]) -> Vec<SilenceEntryView> {
    let mut views: Vec<SilenceEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_entry_view(item));
    }
    views
}

pub fn sum_silence_entry_view(view: &SilenceEntryView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_rate + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_entry("code-874", "label-872");
        let view = to_silence_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
