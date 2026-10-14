use super::*;

use acme_kit::text;

/// How screens and exports show a rule adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleAdjustmentView {
    pub code: String,
    pub name: String,
    pub weight: String,
    pub margin: String,
    pub batch_size: String,
    pub issuer: String,
    pub floor_value: String,
    pub cluster: String,
    pub minimum: String,
    pub version_text: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_rule_adjustment_view(item: &RuleAdjustment) -> RuleAdjustmentView {
    RuleAdjustmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        weight: format!("{:.2}", item.weight),
        margin: format!("{:.2}", item.margin),
        batch_size: item.batch_size.to_string(),
        issuer: item.issuer.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        cluster: item.cluster.clone(),
        minimum: format!("{:.2}", item.minimum),
        version_text: item.version.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_adjustment_views(items: &[RuleAdjustment]) -> Vec<RuleAdjustmentView> {
    let mut views: Vec<RuleAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_adjustment_view(item));
    }
    views
}

pub fn sum_rule_adjustment_view(view: &RuleAdjustmentView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_weight + parsed_margin + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_adjustment("code-170", "name-190");
        let view = to_rule_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
