use super::*;

/// How screens and exports show a approval draft.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalDraftView {
    pub code: String,
    pub label: String,
    pub country: String,
    pub contact_email_text: String,
    pub category: String,
    pub key_id: String,
    pub starts_at: String,
    pub approved: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_approval_draft_view(item: &ApprovalDraft) -> ApprovalDraftView {
    ApprovalDraftView {
        code: item.code.clone(),
        label: item.label.clone(),
        country: item.country.clone(),
        contact_email_text: item.contact_email.clone(),
        category: item.category.clone(),
        key_id: item.key_id.clone(),
        starts_at: item.starts_at.to_string(),
        approved: format!("{}", item.approved),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_draft_views(items: &[ApprovalDraft]) -> Vec<ApprovalDraftView> {
    let mut views: Vec<ApprovalDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_draft_view(item));
    }
    views
}

pub fn sum_approval_draft_view(view: &ApprovalDraftView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_draft("code-380", "label-362");
        let view = to_approval_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
