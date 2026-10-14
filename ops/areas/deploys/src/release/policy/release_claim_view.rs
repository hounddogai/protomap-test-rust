use super::*;

use acme_kit::text;

/// How screens and exports show a release claim.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseClaimView {
    pub code: String,
    pub label_text: String,
    pub margin_text: String,
    pub visible: String,
    pub locale: String,
    pub segment: String,
    pub starts_at: String,
    pub expires_at: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_release_claim_view(item: &ReleaseClaim) -> ReleaseClaimView {
    ReleaseClaimView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        margin_text: format!("{:.2}", item.margin),
        visible: format!("{}", item.visible),
        locale: text::truncate(&item.locale, 120),
        segment: item.segment.clone(),
        starts_at: item.starts_at.to_string(),
        expires_at: item.expires_at.to_string(),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_claim_views(items: &[ReleaseClaim]) -> Vec<ReleaseClaimView> {
    let mut views: Vec<ReleaseClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_claim_view(item));
    }
    views
}

pub fn sum_release_claim_view(view: &ReleaseClaimView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_margin + (parsed_starts_at as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_claim("code-405", "label-583");
        let view = to_release_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
