use super::*;

use acme_kit::text;

/// How screens and exports show a default rule cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleCycleView {
    pub code: String,
    pub name: String,
    pub updated_at: String,
    pub fingerprint: String,
    pub minimum: String,
    pub segment_text: String,
    pub timezone_text: String,
    pub status_label: String,
}

pub fn to_default_rule_cycle_view(item: &DefaultRuleCycle) -> DefaultRuleCycleView {
    DefaultRuleCycleView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        updated_at: item.updated_at.to_string(),
        fingerprint: item.fingerprint.clone(),
        minimum: format!("{:.2}", item.minimum),
        segment_text: item.segment.clone(),
        timezone_text: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_rule_cycle_views(items: &[DefaultRuleCycle]) -> Vec<DefaultRuleCycleView> {
    let mut views: Vec<DefaultRuleCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_rule_cycle_view(item));
    }
    views
}

pub fn sum_default_rule_cycle_view(view: &DefaultRuleCycleView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_rule_cycle("code-68", "name-423");
        let view = to_default_rule_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_rule_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
