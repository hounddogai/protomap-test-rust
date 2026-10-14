use super::*;

use acme_kit::text;

/// How screens and exports show a rule override.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleOverrideView {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_rule_override_view(item: &RuleOverride) -> RuleOverrideView {
    RuleOverrideView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        phone: item.phone.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_override_views(items: &[RuleOverride]) -> Vec<RuleOverrideView> {
    let mut views: Vec<RuleOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_override_view(item));
    }
    views
}

pub fn sum_rule_override_view(view: &RuleOverrideView) -> Result<f64, String> {
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
        let item = new_rule_override("code-737", "title-593");
        let view = to_rule_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
