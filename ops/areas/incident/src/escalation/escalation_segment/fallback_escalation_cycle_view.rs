use super::*;

use acme_kit::text;

/// How screens and exports show a fallback escalation cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEscalationCycleView {
    pub code_text: String,
    pub label_text: String,
    pub ceiling_text: String,
    pub threshold: String,
    pub external_ref_text: String,
    pub weight: String,
    pub tenant_id: String,
    pub rotation_days: String,
    pub title_text: String,
    pub contact_email: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_fallback_escalation_cycle_view(item: &FallbackEscalationCycle) -> FallbackEscalationCycleView {
    FallbackEscalationCycleView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        ceiling_text: format!("{:.2}", item.ceiling),
        threshold: format!("{:.2}", item.threshold),
        external_ref_text: item.external_ref.clone(),
        weight: format!("{:.2}", item.weight),
        tenant_id: item.tenant_id.clone(),
        rotation_days: item.rotation_days.to_string(),
        title_text: item.title.clone(),
        contact_email: item.contact_email.clone(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_escalation_cycle_views(items: &[FallbackEscalationCycle]) -> Vec<FallbackEscalationCycleView> {
    let mut views: Vec<FallbackEscalationCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_escalation_cycle_view(item));
    }
    views
}

pub fn sum_fallback_escalation_cycle_view(view: &FallbackEscalationCycleView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_ceiling + parsed_threshold + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_escalation_cycle("code-172", "label-122");
        let view = to_fallback_escalation_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_escalation_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
