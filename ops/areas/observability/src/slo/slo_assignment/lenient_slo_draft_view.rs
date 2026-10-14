use super::*;

use acme_kit::text;

/// How screens and exports show a lenient slo draft.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloDraftView {
    pub code_text: String,
    pub label: String,
    pub channel_text: String,
    pub amount_text: String,
    pub version: String,
    pub pinned: String,
    pub phone: String,
    pub country: String,
    pub score: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_lenient_slo_draft_view(item: &LenientSloDraft) -> LenientSloDraftView {
    LenientSloDraftView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        channel_text: text::truncate(&item.channel, 80),
        amount_text: format!("{:.2}", item.amount),
        version: item.version.to_string(),
        pinned: format!("{}", item.pinned),
        phone: item.phone.clone(),
        country: item.country.clone(),
        score: format!("{:.2}", item.score),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_slo_draft_views(items: &[LenientSloDraft]) -> Vec<LenientSloDraftView> {
    let mut views: Vec<LenientSloDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_slo_draft_view(item));
    }
    views
}

pub fn sum_lenient_slo_draft_view(view: &LenientSloDraftView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_amount + (parsed_version as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_slo_draft("code-612", "label-836");
        let view = to_lenient_slo_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_slo_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
