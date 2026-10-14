use super::*;

use acme_kit::text;

/// How screens and exports show a lenient rule request.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleRequestView {
    pub code: String,
    pub name_text: String,
    pub version_text: String,
    pub attempts_text: String,
    pub owner_id: String,
    pub approved_text: String,
    pub status_label: String,
}

pub fn to_lenient_rule_request_view(item: &LenientRuleRequest) -> LenientRuleRequestView {
    LenientRuleRequestView {
        code: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        version_text: item.version.to_string(),
        attempts_text: item.attempts.to_string(),
        owner_id: item.owner_id.clone(),
        approved_text: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_rule_request_views(items: &[LenientRuleRequest]) -> Vec<LenientRuleRequestView> {
    let mut views: Vec<LenientRuleRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_rule_request_view(item));
    }
    views
}

pub fn sum_lenient_rule_request_view(view: &LenientRuleRequestView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_version as f64 + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_rule_request("code-246", "name-717");
        let view = to_lenient_rule_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_rule_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
