use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled budget burn notice.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBudgetBurnNoticeView {
    pub code: String,
    pub name: String,
    pub reviewer_text: String,
    pub contact_email_text: String,
    pub region: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_scheduled_budget_burn_notice_view(item: &ScheduledBudgetBurnNotice) -> ScheduledBudgetBurnNoticeView {
    ScheduledBudgetBurnNoticeView {
        code: item.code.clone(),
        name: item.name.clone(),
        reviewer_text: text::truncate(&item.reviewer, 64),
        contact_email_text: text::truncate(&item.contact_email, 40),
        region: text::truncate(&item.region, 40),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_budget_burn_notice_views(
    items: &[ScheduledBudgetBurnNotice],
) -> Vec<ScheduledBudgetBurnNoticeView> {
    let mut views: Vec<ScheduledBudgetBurnNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_budget_burn_notice_view(item));
    }
    views
}

pub fn sum_scheduled_budget_burn_notice_view(view: &ScheduledBudgetBurnNoticeView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}
