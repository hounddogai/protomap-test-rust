use super::*;

use acme_kit::text;

/// How screens and exports show a sink entry.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkEntryView {
    pub code_text: String,
    pub name_text: String,
    pub rate_text: String,
    pub external_ref_text: String,
    pub fingerprint: String,
    pub expires_at: String,
    pub archived: String,
    pub audience: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_sink_entry_view(item: &SinkEntry) -> SinkEntryView {
    SinkEntryView {
        code_text: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        rate_text: format!("{:.2}", item.rate),
        external_ref_text: item.external_ref.clone(),
        fingerprint: item.fingerprint.clone(),
        expires_at: item.expires_at.to_string(),
        archived: format!("{}", item.archived),
        audience: text::truncate(&item.audience, 120),
        checksum: text::truncate(&item.checksum, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_entry_views(items: &[SinkEntry]) -> Vec<SinkEntryView> {
    let mut views: Vec<SinkEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_entry_view(item));
    }
    views
}

pub fn sum_sink_entry_view(view: &SinkEntryView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_rate + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_sink_entry("code-136", "name-747");
        let view = to_sink_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_sink_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
