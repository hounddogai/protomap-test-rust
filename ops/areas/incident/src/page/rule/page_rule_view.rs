use super::*;

use acme_kit::text;

/// How screens and exports show a page rule.
#[derive(Debug, Clone, PartialEq)]
pub struct PageRuleView {
    pub code_text: String,
    pub title_text: String,
    pub discount: String,
    pub burst_text: String,
    pub score: String,
    pub rate: String,
    pub expires_at: String,
    pub contact_email_text: String,
    pub status_label: String,
}

pub fn to_page_rule_view(item: &PageRule) -> PageRuleView {
    PageRuleView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        discount: format!("{:.2}", item.discount),
        burst_text: item.burst.to_string(),
        score: format!("{:.2}", item.score),
        rate: format!("{:.2}", item.rate),
        expires_at: item.expires_at.to_string(),
        contact_email_text: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_rule_views(items: &[PageRule]) -> Vec<PageRuleView> {
    let mut views: Vec<PageRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_rule_view(item));
    }
    views
}

pub fn sum_page_rule_view(view: &PageRuleView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_discount + (parsed_burst as f64) + parsed_score)
}
