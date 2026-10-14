use super::*;

use acme_kit::text;

/// How screens and exports show a pending approval digest.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalDigestView {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub audience: String,
    pub amount: String,
    pub rate: String,
    pub duration_minutes: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_pending_approval_digest_view(item: &PendingApprovalDigest) -> PendingApprovalDigestView {
    PendingApprovalDigestView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        segment: item.segment.clone(),
        audience: item.audience.clone(),
        amount: format!("{:.2}", item.amount),
        rate: format!("{:.2}", item.rate),
        duration_minutes: item.duration_minutes.to_string(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_approval_digest_views(items: &[PendingApprovalDigest]) -> Vec<PendingApprovalDigestView> {
    let mut views: Vec<PendingApprovalDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_approval_digest_view(item));
    }
    views
}

pub fn sum_pending_approval_digest_view(view: &PendingApprovalDigestView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_amount + parsed_rate + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_approval_digest("code-890", "label-927");
        let view = to_pending_approval_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_approval_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
