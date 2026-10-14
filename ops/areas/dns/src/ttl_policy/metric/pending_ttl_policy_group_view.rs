use super::*;

use acme_kit::text;

/// How screens and exports show a pending ttl policy group.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyGroupView {
    pub code: String,
    pub title: String,
    pub sequence: String,
    pub ratio: String,
    pub external_ref_text: String,
    pub phone: String,
    pub score: String,
    pub discount: String,
    pub burst_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_pending_ttl_policy_group_view(item: &PendingTtlPolicyGroup) -> PendingTtlPolicyGroupView {
    PendingTtlPolicyGroupView {
        code: item.code.clone(),
        title: item.title.clone(),
        sequence: item.sequence.to_string(),
        ratio: format!("{:.2}", item.ratio),
        external_ref_text: item.external_ref.clone(),
        phone: text::truncate(&item.phone, 80),
        score: format!("{:.2}", item.score),
        discount: format!("{:.2}", item.discount),
        burst_text: item.burst.to_string(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_ttl_policy_group_views(items: &[PendingTtlPolicyGroup]) -> Vec<PendingTtlPolicyGroupView> {
    let mut views: Vec<PendingTtlPolicyGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_ttl_policy_group_view(item));
    }
    views
}

pub fn sum_pending_ttl_policy_group_view(view: &PendingTtlPolicyGroupView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_ratio + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_ttl_policy_group("code-501", "title-855");
        let view = to_pending_ttl_policy_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_ttl_policy_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
