use super::*;

use acme_kit::text;

/// How screens and exports show a page variance.
#[derive(Debug, Clone, PartialEq)]
pub struct PageVarianceView {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub active_text: String,
    pub pinned_text: String,
    pub fingerprint_text: String,
    pub segment_text: String,
    pub summary: String,
    pub notes_text: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_page_variance_view(item: &PageVariance) -> PageVarianceView {
    PageVarianceView {
        code: item.code.clone(),
        title: item.title.clone(),
        owner_id: text::truncate(&item.owner_id, 80),
        active_text: format!("{}", item.active),
        pinned_text: format!("{}", item.pinned),
        fingerprint_text: item.fingerprint.clone(),
        segment_text: text::truncate(&item.segment, 40),
        summary: item.summary.clone(),
        notes_text: text::truncate(&item.notes, 120),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_variance_views(items: &[PageVariance]) -> Vec<PageVarianceView> {
    let mut views: Vec<PageVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_variance_view(item));
    }
    views
}

pub fn sum_page_variance_view(view: &PageVarianceView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_page_variance("code-837", "title-710");
        let view = to_page_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_page_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
