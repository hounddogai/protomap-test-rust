use super::*;

use acme_kit::text;

/// How screens and exports show a primary peering tier.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringTierView {
    pub code_text: String,
    pub label: String,
    pub segment: String,
    pub budget_text: String,
    pub channel: String,
    pub discount: String,
    pub key_id_text: String,
    pub phone: String,
    pub maximum: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_primary_peering_tier_view(item: &PrimaryPeeringTier) -> PrimaryPeeringTierView {
    PrimaryPeeringTierView {
        code_text: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 80),
        segment: text::truncate(&item.segment, 40),
        budget_text: format!("{:.2}", item.budget),
        channel: item.channel.clone(),
        discount: format!("{:.2}", item.discount),
        key_id_text: item.key_id.clone(),
        phone: item.phone.clone(),
        maximum: format!("{:.2}", item.maximum),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_peering_tier_views(items: &[PrimaryPeeringTier]) -> Vec<PrimaryPeeringTierView> {
    let mut views: Vec<PrimaryPeeringTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_peering_tier_view(item));
    }
    views
}

pub fn sum_primary_peering_tier_view(view: &PrimaryPeeringTierView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_budget + parsed_discount + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_peering_tier("code-164", "label-859");
        let view = to_primary_peering_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_peering_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
