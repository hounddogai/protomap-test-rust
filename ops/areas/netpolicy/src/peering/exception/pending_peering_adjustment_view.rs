use super::*;

/// How screens and exports show a pending peering adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPeeringAdjustmentView {
    pub code_text: String,
    pub name: String,
    pub margin: String,
    pub cluster_text: String,
    pub notes: String,
    pub amount: String,
    pub discount: String,
    pub key_id_text: String,
    pub status_label: String,
}

pub fn to_pending_peering_adjustment_view(item: &PendingPeeringAdjustment) -> PendingPeeringAdjustmentView {
    PendingPeeringAdjustmentView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        margin: format!("{:.2}", item.margin),
        cluster_text: item.cluster.clone(),
        notes: item.notes.clone(),
        amount: format!("{:.2}", item.amount),
        discount: format!("{:.2}", item.discount),
        key_id_text: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_peering_adjustment_views(items: &[PendingPeeringAdjustment]) -> Vec<PendingPeeringAdjustmentView> {
    let mut views: Vec<PendingPeeringAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_peering_adjustment_view(item));
    }
    views
}

pub fn sum_pending_peering_adjustment_view(view: &PendingPeeringAdjustmentView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_margin + parsed_amount + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_peering_adjustment("code-501", "name-576");
        let view = to_pending_peering_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_peering_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
