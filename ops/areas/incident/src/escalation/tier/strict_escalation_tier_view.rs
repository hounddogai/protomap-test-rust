use super::*;

/// How screens and exports show a strict escalation tier.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationTierView {
    pub code_text: String,
    pub label: String,
    pub revision_text: String,
    pub updated_at: String,
    pub segment: String,
    pub description_text: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_strict_escalation_tier_view(item: &StrictEscalationTier) -> StrictEscalationTierView {
    StrictEscalationTierView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        revision_text: item.revision.to_string(),
        updated_at: item.updated_at.to_string(),
        segment: item.segment.clone(),
        description_text: item.description.clone(),
        owner_id: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_escalation_tier_views(items: &[StrictEscalationTier]) -> Vec<StrictEscalationTierView> {
    let mut views: Vec<StrictEscalationTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_escalation_tier_view(item));
    }
    views
}

pub fn sum_strict_escalation_tier_view(view: &StrictEscalationTierView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_escalation_tier("code-298", "label-835");
        let view = to_strict_escalation_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_escalation_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
