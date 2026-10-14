use super::*;

use acme_kit::text;

/// How screens and exports show a primary approval notice.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryApprovalNoticeView {
    pub code_text: String,
    pub title: String,
    pub score: String,
    pub key_id: String,
    pub name: String,
    pub budget_text: String,
    pub created_at: String,
    pub timezone_text: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_primary_approval_notice_view(item: &PrimaryApprovalNotice) -> PrimaryApprovalNoticeView {
    PrimaryApprovalNoticeView {
        code_text: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 40),
        score: format!("{:.2}", item.score),
        key_id: item.key_id.clone(),
        name: item.name.clone(),
        budget_text: format!("{:.2}", item.budget),
        created_at: item.created_at.to_string(),
        timezone_text: item.timezone.clone(),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_approval_notice_views(items: &[PrimaryApprovalNotice]) -> Vec<PrimaryApprovalNoticeView> {
    let mut views: Vec<PrimaryApprovalNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_approval_notice_view(item));
    }
    views
}

pub fn sum_primary_approval_notice_view(view: &PrimaryApprovalNoticeView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_score + parsed_budget + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_approval_notice("code-253", "title-162");
        let view = to_primary_approval_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_approval_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
