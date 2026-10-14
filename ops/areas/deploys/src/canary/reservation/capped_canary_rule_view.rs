use super::*;

use acme_kit::text;

/// How screens and exports show a capped canary rule.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedCanaryRuleView {
    pub code_text: String,
    pub title: String,
    pub ttl_seconds: String,
    pub ceiling: String,
    pub description: String,
    pub batch_size: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_capped_canary_rule_view(item: &CappedCanaryRule) -> CappedCanaryRuleView {
    CappedCanaryRuleView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        description: text::truncate(&item.description, 40),
        batch_size: item.batch_size.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_canary_rule_views(items: &[CappedCanaryRule]) -> Vec<CappedCanaryRuleView> {
    let mut views: Vec<CappedCanaryRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_canary_rule_view(item));
    }
    views
}

pub fn sum_capped_canary_rule_view(view: &CappedCanaryRuleView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_ceiling + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_canary_rule("code-460", "title-339");
        let view = to_capped_canary_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_canary_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
