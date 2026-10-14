use super::*;

use acme_kit::text;

/// How screens and exports show a lenient sink draft.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkDraftView {
    pub code: String,
    pub title_text: String,
    pub expires_at_text: String,
    pub ratio: String,
    pub minimum_text: String,
    pub summary: String,
    pub source: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_lenient_sink_draft_view(item: &LenientSinkDraft) -> LenientSinkDraftView {
    LenientSinkDraftView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        expires_at_text: item.expires_at.to_string(),
        ratio: format!("{:.2}", item.ratio),
        minimum_text: format!("{:.2}", item.minimum),
        summary: item.summary.clone(),
        source: item.source.clone(),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_sink_draft_views(items: &[LenientSinkDraft]) -> Vec<LenientSinkDraftView> {
    let mut views: Vec<LenientSinkDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_sink_draft_view(item));
    }
    views
}

pub fn sum_lenient_sink_draft_view(view: &LenientSinkDraftView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_ratio + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_sink_draft("code-839", "title-256");
        let view = to_lenient_sink_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_sink_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
