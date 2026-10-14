use super::*;

use acme_kit::text;

/// How screens and exports show a pending plan quota.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanQuotaView {
    pub code: String,
    pub label_text: String,
    pub cipher_text: String,
    pub namespace_text: String,
    pub reviewer_text: String,
    pub score: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_pending_plan_quota_view(item: &PendingPlanQuota) -> PendingPlanQuotaView {
    PendingPlanQuotaView {
        code: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        cipher_text: text::truncate(&item.cipher, 40),
        namespace_text: item.namespace.clone(),
        reviewer_text: text::truncate(&item.reviewer, 120),
        score: format!("{:.2}", item.score),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_plan_quota_views(items: &[PendingPlanQuota]) -> Vec<PendingPlanQuotaView> {
    let mut views: Vec<PendingPlanQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_plan_quota_view(item));
    }
    views
}

pub fn sum_pending_plan_quota_view(view: &PendingPlanQuotaView) -> Result<f64, String> {
    let parsed_score = view
        .score
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
        let item = new_pending_plan_quota("code-806", "label-532");
        let view = to_pending_plan_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_plan_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
