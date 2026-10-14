use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled quote agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuoteAgreementView {
    pub code: String,
    pub title_text: String,
    pub starts_at_text: String,
    pub rate_text: String,
    pub visible_text: String,
    pub status_label: String,
}

pub fn to_scheduled_quote_agreement_view(item: &ScheduledQuoteAgreement) -> ScheduledQuoteAgreementView {
    ScheduledQuoteAgreementView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        starts_at_text: item.starts_at.to_string(),
        rate_text: format!("{:.2}", item.rate),
        visible_text: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_quote_agreement_views(items: &[ScheduledQuoteAgreement]) -> Vec<ScheduledQuoteAgreementView> {
    let mut views: Vec<ScheduledQuoteAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_quote_agreement_view(item));
    }
    views
}

pub fn sum_scheduled_quote_agreement_view(view: &ScheduledQuoteAgreementView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_quote_agreement("code-535", "title-524");
        let view = to_scheduled_quote_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_quote_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
