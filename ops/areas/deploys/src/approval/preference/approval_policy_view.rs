use super::*;

use acme_kit::text;

/// How screens and exports show a approval policy.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPolicyView {
    pub code_text: String,
    pub label: String,
    pub owner_id: String,
    pub batch_size: String,
    pub score_text: String,
    pub sequence: String,
    pub created_at: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_approval_policy_view(item: &ApprovalPolicy) -> ApprovalPolicyView {
    ApprovalPolicyView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 80),
        owner_id: item.owner_id.clone(),
        batch_size: item.batch_size.to_string(),
        score_text: format!("{:.2}", item.score),
        sequence: item.sequence.to_string(),
        created_at: item.created_at.to_string(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_policy_views(items: &[ApprovalPolicy]) -> Vec<ApprovalPolicyView> {
    let mut views: Vec<ApprovalPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_policy_view(item));
    }
    views
}

pub fn sum_approval_policy_view(view: &ApprovalPolicyView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_score + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_policy("code-114", "label-763");
        let view = to_approval_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
