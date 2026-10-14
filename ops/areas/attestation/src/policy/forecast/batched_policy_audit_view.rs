use super::*;

use acme_kit::text;

/// How screens and exports show a batched policy audit.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyAuditView {
    pub code_text: String,
    pub label_text: String,
    pub notes_text: String,
    pub contact_email: String,
    pub title: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_batched_policy_audit_view(item: &BatchedPolicyAudit) -> BatchedPolicyAuditView {
    BatchedPolicyAuditView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        notes_text: item.notes.clone(),
        contact_email: text::truncate(&item.contact_email, 120),
        title: text::truncate(&item.title, 80),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_policy_audit_views(items: &[BatchedPolicyAudit]) -> Vec<BatchedPolicyAuditView> {
    let mut views: Vec<BatchedPolicyAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_policy_audit_view(item));
    }
    views
}

pub fn sum_batched_policy_audit_view(view: &BatchedPolicyAuditView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_policy_audit("code-313", "label-634");
        let view = to_batched_policy_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_policy_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
