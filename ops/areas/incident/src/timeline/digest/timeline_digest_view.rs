use super::*;

use acme_kit::text;

/// How screens and exports show a timeline digest.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineDigestView {
    pub code: String,
    pub title: String,
    pub country: String,
    pub label: String,
    pub threshold: String,
    pub quantity: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_timeline_digest_view(item: &TimelineDigest) -> TimelineDigestView {
    TimelineDigestView {
        code: item.code.clone(),
        title: item.title.clone(),
        country: text::truncate(&item.country, 40),
        label: item.label.clone(),
        threshold: format!("{:.2}", item.threshold),
        quantity: item.quantity.to_string(),
        notes: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_digest_views(items: &[TimelineDigest]) -> Vec<TimelineDigestView> {
    let mut views: Vec<TimelineDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_digest_view(item));
    }
    views
}

pub fn sum_timeline_digest_view(view: &TimelineDigestView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_threshold + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_digest("code-707", "title-524");
        let view = to_timeline_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
