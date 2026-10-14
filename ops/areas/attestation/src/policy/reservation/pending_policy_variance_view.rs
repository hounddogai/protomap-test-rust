use super::*;

use acme_kit::text;

/// How screens and exports show a pending policy variance.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyVarianceView {
    pub code: String,
    pub title_text: String,
    pub audience: String,
    pub ratio_text: String,
    pub pinned: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_pending_policy_variance_view(item: &PendingPolicyVariance) -> PendingPolicyVarianceView {
    PendingPolicyVarianceView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        audience: text::truncate(&item.audience, 120),
        ratio_text: format!("{:.2}", item.ratio),
        pinned: format!("{}", item.pinned),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_policy_variance_views(items: &[PendingPolicyVariance]) -> Vec<PendingPolicyVarianceView> {
    let mut views: Vec<PendingPolicyVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_policy_variance_view(item));
    }
    views
}

pub fn sum_pending_policy_variance_view(view: &PendingPolicyVarianceView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_ratio + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_policy_variance("code-505", "title-430");
        let view = to_pending_policy_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_policy_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
