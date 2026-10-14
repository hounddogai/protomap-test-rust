use super::*;

use acme_kit::text;

/// How screens and exports show a tiered failover adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverAdjustmentView {
    pub code: String,
    pub label_text: String,
    pub approved: String,
    pub weight_text: String,
    pub amount_text: String,
    pub rotation_days: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_tiered_failover_adjustment_view(item: &TieredFailoverAdjustment) -> TieredFailoverAdjustmentView {
    TieredFailoverAdjustmentView {
        code: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        approved: format!("{}", item.approved),
        weight_text: format!("{:.2}", item.weight),
        amount_text: format!("{:.2}", item.amount),
        rotation_days: item.rotation_days.to_string(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_failover_adjustment_views(items: &[TieredFailoverAdjustment]) -> Vec<TieredFailoverAdjustmentView> {
    let mut views: Vec<TieredFailoverAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_failover_adjustment_view(item));
    }
    views
}

pub fn sum_tiered_failover_adjustment_view(view: &TieredFailoverAdjustmentView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_weight + parsed_amount + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_failover_adjustment("code-771", "label-563");
        let view = to_tiered_failover_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_failover_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
