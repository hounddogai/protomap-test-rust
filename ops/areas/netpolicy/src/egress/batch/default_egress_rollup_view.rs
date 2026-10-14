use super::*;

use acme_kit::text;

/// How screens and exports show a default egress rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressRollupView {
    pub code: String,
    pub label_text: String,
    pub scope: String,
    pub minimum: String,
    pub amount_text: String,
    pub maximum: String,
    pub expires_at: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_default_egress_rollup_view(item: &DefaultEgressRollup) -> DefaultEgressRollupView {
    DefaultEgressRollupView {
        code: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        scope: text::truncate(&item.scope, 120),
        minimum: format!("{:.2}", item.minimum),
        amount_text: format!("{:.2}", item.amount),
        maximum: format!("{:.2}", item.maximum),
        expires_at: item.expires_at.to_string(),
        checksum: text::truncate(&item.checksum, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_egress_rollup_views(items: &[DefaultEgressRollup]) -> Vec<DefaultEgressRollupView> {
    let mut views: Vec<DefaultEgressRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_egress_rollup_view(item));
    }
    views
}

pub fn sum_default_egress_rollup_view(view: &DefaultEgressRollupView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_minimum + parsed_amount + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_egress_rollup("code-623", "label-676");
        let view = to_default_egress_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_egress_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
