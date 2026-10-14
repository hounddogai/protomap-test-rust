use super::*;

use acme_kit::text;

/// How screens and exports show a rule request.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleRequestView {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub unit_text: String,
    pub fingerprint: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_rule_request_view(item: &RuleRequest) -> RuleRequestView {
    RuleRequestView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        contact_email: item.contact_email.clone(),
        unit_text: item.unit.clone(),
        fingerprint: item.fingerprint.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_request_views(items: &[RuleRequest]) -> Vec<RuleRequestView> {
    let mut views: Vec<RuleRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_request_view(item));
    }
    views
}

pub fn sum_rule_request_view(view: &RuleRequestView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_request("code-734", "label-796");
        let view = to_rule_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
