use super::*;

use acme_kit::text;

/// How screens and exports show a slo digest.
#[derive(Debug, Clone, PartialEq)]
pub struct SloDigestView {
    pub code: String,
    pub label: String,
    pub pinned: String,
    pub archived: String,
    pub active: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_slo_digest_view(item: &SloDigest) -> SloDigestView {
    SloDigestView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        pinned: format!("{}", item.pinned),
        archived: format!("{}", item.archived),
        active: format!("{}", item.active),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_digest_views(items: &[SloDigest]) -> Vec<SloDigestView> {
    let mut views: Vec<SloDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_digest_view(item));
    }
    views
}

pub fn sum_slo_digest_view(view: &SloDigestView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_digest("code-730", "label-596");
        let view = to_slo_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
