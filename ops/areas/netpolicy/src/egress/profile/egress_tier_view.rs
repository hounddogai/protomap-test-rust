use super::*;

use acme_kit::text;

/// How screens and exports show a egress tier.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressTierView {
    pub code_text: String,
    pub name: String,
    pub duration_minutes_text: String,
    pub starts_at: String,
    pub quantity_text: String,
    pub region: String,
    pub contact_email: String,
    pub margin_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_egress_tier_view(item: &EgressTier) -> EgressTierView {
    EgressTierView {
        code_text: text::truncate(&item.code, 80),
        name: item.name.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        starts_at: item.starts_at.to_string(),
        quantity_text: item.quantity.to_string(),
        region: item.region.clone(),
        contact_email: item.contact_email.clone(),
        margin_text: format!("{:.2}", item.margin),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_tier_views(items: &[EgressTier]) -> Vec<EgressTierView> {
    let mut views: Vec<EgressTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_tier_view(item));
    }
    views
}

pub fn sum_egress_tier_view(view: &EgressTierView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_starts_at as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_tier("code-465", "name-130");
        let view = to_egress_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
