use super::*;

/// How screens and exports show a batched escalation tier.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEscalationTierView {
    pub code: String,
    pub label_text: String,
    pub retries: String,
    pub updated_at_text: String,
    pub ceiling_text: String,
    pub maximum_text: String,
    pub minimum_text: String,
    pub rate_text: String,
    pub starts_at_text: String,
    pub key_id_text: String,
    pub status_label: String,
}

pub fn to_batched_escalation_tier_view(item: &BatchedEscalationTier) -> BatchedEscalationTierView {
    BatchedEscalationTierView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        retries: item.retries.to_string(),
        updated_at_text: item.updated_at.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        maximum_text: format!("{:.2}", item.maximum),
        minimum_text: format!("{:.2}", item.minimum),
        rate_text: format!("{:.2}", item.rate),
        starts_at_text: item.starts_at.to_string(),
        key_id_text: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_escalation_tier_views(items: &[BatchedEscalationTier]) -> Vec<BatchedEscalationTierView> {
    let mut views: Vec<BatchedEscalationTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_escalation_tier_view(item));
    }
    views
}

pub fn sum_batched_escalation_tier_view(view: &BatchedEscalationTierView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_updated_at as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_escalation_tier("code-969", "label-34");
        let view = to_batched_escalation_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_escalation_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
