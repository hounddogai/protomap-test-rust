use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled quote group.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuoteGroupView {
    pub code: String,
    pub label_text: String,
    pub expires_at_text: String,
    pub owner_id: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_scheduled_quote_group_view(item: &ScheduledQuoteGroup) -> ScheduledQuoteGroupView {
    ScheduledQuoteGroupView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        expires_at_text: item.expires_at.to_string(),
        owner_id: text::truncate(&item.owner_id, 64),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_quote_group_views(items: &[ScheduledQuoteGroup]) -> Vec<ScheduledQuoteGroupView> {
    let mut views: Vec<ScheduledQuoteGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_quote_group_view(item));
    }
    views
}

pub fn sum_scheduled_quote_group_view(view: &ScheduledQuoteGroupView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_quote_group("code-362", "label-321");
        let view = to_scheduled_quote_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_quote_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
