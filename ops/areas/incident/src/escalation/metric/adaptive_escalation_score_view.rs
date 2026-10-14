use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive escalation score.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEscalationScoreView {
    pub code_text: String,
    pub title: String,
    pub cipher: String,
    pub amount: String,
    pub label: String,
    pub locale_text: String,
    pub rate_text: String,
    pub status_label: String,
}

pub fn to_adaptive_escalation_score_view(item: &AdaptiveEscalationScore) -> AdaptiveEscalationScoreView {
    AdaptiveEscalationScoreView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        cipher: item.cipher.clone(),
        amount: format!("{:.2}", item.amount),
        label: item.label.clone(),
        locale_text: text::truncate(&item.locale, 120),
        rate_text: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_escalation_score_views(items: &[AdaptiveEscalationScore]) -> Vec<AdaptiveEscalationScoreView> {
    let mut views: Vec<AdaptiveEscalationScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_escalation_score_view(item));
    }
    views
}

pub fn sum_adaptive_escalation_score_view(view: &AdaptiveEscalationScoreView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_amount + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_escalation_score("code-955", "title-447");
        let view = to_adaptive_escalation_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_escalation_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
