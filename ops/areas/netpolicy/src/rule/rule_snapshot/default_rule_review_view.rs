use super::*;

/// How screens and exports show a default rule review.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleReviewView {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub threshold_text: String,
    pub starts_at_text: String,
    pub contact_email: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_default_rule_review_view(item: &DefaultRuleReview) -> DefaultRuleReviewView {
    DefaultRuleReviewView {
        code: item.code.clone(),
        label: item.label.clone(),
        unit: item.unit.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        starts_at_text: item.starts_at.to_string(),
        contact_email: item.contact_email.clone(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_rule_review_views(items: &[DefaultRuleReview]) -> Vec<DefaultRuleReviewView> {
    let mut views: Vec<DefaultRuleReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_rule_review_view(item));
    }
    views
}

pub fn sum_default_rule_review_view(view: &DefaultRuleReviewView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_threshold + (parsed_starts_at as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_rule_review("code-403", "label-670");
        let view = to_default_rule_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_rule_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
