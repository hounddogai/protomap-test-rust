use super::*;

use acme_kit::text;

/// How screens and exports show a verifier entry.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierEntryView {
    pub code: String,
    pub label_text: String,
    pub fingerprint_text: String,
    pub source: String,
    pub summary: String,
    pub active: String,
    pub external_ref: String,
    pub region: String,
    pub weight: String,
    pub locale: String,
    pub segment_text: String,
    pub status_label: String,
}

pub fn to_verifier_entry_view(item: &VerifierEntry) -> VerifierEntryView {
    VerifierEntryView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        fingerprint_text: item.fingerprint.clone(),
        source: item.source.clone(),
        summary: item.summary.clone(),
        active: format!("{}", item.active),
        external_ref: text::truncate(&item.external_ref, 40),
        region: item.region.clone(),
        weight: format!("{:.2}", item.weight),
        locale: item.locale.clone(),
        segment_text: text::truncate(&item.segment, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_entry_views(items: &[VerifierEntry]) -> Vec<VerifierEntryView> {
    let mut views: Vec<VerifierEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_entry_view(item));
    }
    views
}

pub fn sum_verifier_entry_view(view: &VerifierEntryView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_entry("code-880", "label-871");
        let view = to_verifier_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
