use super::*;

use acme_kit::text;

/// How screens and exports show a peering adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAdjustmentView {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub cipher: String,
    pub owner_id: String,
    pub issuer: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_peering_adjustment_view(item: &PeeringAdjustment) -> PeeringAdjustmentView {
    PeeringAdjustmentView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        external_ref: item.external_ref.clone(),
        cipher: text::truncate(&item.cipher, 80),
        owner_id: text::truncate(&item.owner_id, 64),
        issuer: text::truncate(&item.issuer, 80),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_adjustment_views(items: &[PeeringAdjustment]) -> Vec<PeeringAdjustmentView> {
    let mut views: Vec<PeeringAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_adjustment_view(item));
    }
    views
}

pub fn sum_peering_adjustment_view(view: &PeeringAdjustmentView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_adjustment("code-295", "name-68");
        let view = to_peering_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
