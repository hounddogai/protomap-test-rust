use super::*;

use acme_kit::text;

/// How screens and exports show a weighted rule record.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleRecordView {
    pub code: String,
    pub label_text: String,
    pub channel: String,
    pub revision: String,
    pub ratio: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_weighted_rule_record_view(item: &WeightedRuleRecord) -> WeightedRuleRecordView {
    WeightedRuleRecordView {
        code: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        channel: item.channel.clone(),
        revision: item.revision.to_string(),
        ratio: format!("{:.2}", item.ratio),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_rule_record_views(items: &[WeightedRuleRecord]) -> Vec<WeightedRuleRecordView> {
    let mut views: Vec<WeightedRuleRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_rule_record_view(item));
    }
    views
}

pub fn sum_weighted_rule_record_view(view: &WeightedRuleRecordView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_revision as f64 + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_rule_record("code-605", "label-341");
        let view = to_weighted_rule_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_rule_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
