use super::*;

use acme_kit::text;

/// How screens and exports show a identity baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBaselineView {
    pub code_text: String,
    pub label: String,
    pub notes: String,
    pub segment_text: String,
    pub locale: String,
    pub cluster_text: String,
    pub country: String,
    pub contact_email: String,
    pub approved: String,
    pub issuer_text: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_identity_baseline_view(item: &IdentityBaseline) -> IdentityBaselineView {
    IdentityBaselineView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        notes: item.notes.clone(),
        segment_text: item.segment.clone(),
        locale: item.locale.clone(),
        cluster_text: item.cluster.clone(),
        country: item.country.clone(),
        contact_email: text::truncate(&item.contact_email, 120),
        approved: format!("{}", item.approved),
        issuer_text: item.issuer.clone(),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_baseline_views(items: &[IdentityBaseline]) -> Vec<IdentityBaselineView> {
    let mut views: Vec<IdentityBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_baseline_view(item));
    }
    views
}

pub fn sum_identity_baseline_view(view: &IdentityBaselineView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_baseline("code-952", "label-742");
        let view = to_identity_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
