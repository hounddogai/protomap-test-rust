use super::*;

/// How screens and exports show a default policy tier.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyTierView {
    pub code: String,
    pub label: String,
    pub pinned: String,
    pub floor_value: String,
    pub sequence_text: String,
    pub grace_days_text: String,
    pub contact_email_text: String,
    pub batch_size: String,
    pub locale: String,
    pub capacity: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_default_policy_tier_view(item: &DefaultPolicyTier) -> DefaultPolicyTierView {
    DefaultPolicyTierView {
        code: item.code.clone(),
        label: item.label.clone(),
        pinned: format!("{}", item.pinned),
        floor_value: format!("{:.2}", item.floor_value),
        sequence_text: item.sequence.to_string(),
        grace_days_text: item.grace_days.to_string(),
        contact_email_text: item.contact_email.clone(),
        batch_size: item.batch_size.to_string(),
        locale: item.locale.clone(),
        capacity: item.capacity.to_string(),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_policy_tier_views(items: &[DefaultPolicyTier]) -> Vec<DefaultPolicyTierView> {
    let mut views: Vec<DefaultPolicyTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_policy_tier_view(item));
    }
    views
}

pub fn sum_default_policy_tier_view(view: &DefaultPolicyTierView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_floor_value + (parsed_sequence as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_policy_tier("code-487", "label-285");
        let view = to_default_policy_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_policy_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
