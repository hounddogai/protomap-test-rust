use super::*;

/// How screens and exports show a rolling peering tier.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringTierView {
    pub code: String,
    pub name_text: String,
    pub locale: String,
    pub description_text: String,
    pub attempts: String,
    pub unit_text: String,
    pub phone_text: String,
    pub floor_value: String,
    pub maximum_text: String,
    pub created_at_text: String,
    pub status_label: String,
}

pub fn to_rolling_peering_tier_view(item: &RollingPeeringTier) -> RollingPeeringTierView {
    RollingPeeringTierView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        locale: item.locale.clone(),
        description_text: item.description.clone(),
        attempts: item.attempts.to_string(),
        unit_text: item.unit.clone(),
        phone_text: item.phone.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        maximum_text: format!("{:.2}", item.maximum),
        created_at_text: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_peering_tier_views(items: &[RollingPeeringTier]) -> Vec<RollingPeeringTierView> {
    let mut views: Vec<RollingPeeringTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_peering_tier_view(item));
    }
    views
}

pub fn sum_rolling_peering_tier_view(view: &RollingPeeringTierView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_floor_value + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_peering_tier("code-358", "name-782");
        let view = to_rolling_peering_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_peering_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
