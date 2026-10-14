use super::*;

use acme_kit::text;

/// How screens and exports show a lenient identity segment.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientIdentitySegmentView {
    pub code: String,
    pub name: String,
    pub margin: String,
    pub limit_text: String,
    pub cluster: String,
    pub phone: String,
    pub cipher_text: String,
    pub host: String,
    pub updated_at_text: String,
    pub status_label: String,
}

pub fn to_lenient_identity_segment_view(item: &LenientIdentitySegment) -> LenientIdentitySegmentView {
    LenientIdentitySegmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        margin: format!("{:.2}", item.margin),
        limit_text: item.limit.to_string(),
        cluster: text::truncate(&item.cluster, 40),
        phone: item.phone.clone(),
        cipher_text: text::truncate(&item.cipher, 40),
        host: item.host.clone(),
        updated_at_text: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_identity_segment_views(items: &[LenientIdentitySegment]) -> Vec<LenientIdentitySegmentView> {
    let mut views: Vec<LenientIdentitySegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_identity_segment_view(item));
    }
    views
}

pub fn sum_lenient_identity_segment_view(view: &LenientIdentitySegmentView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_margin + (parsed_limit as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_identity_segment("code-870", "name-744");
        let view = to_lenient_identity_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_identity_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
