use super::*;

use acme_kit::text;

/// How screens and exports show a escalation template.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationTemplateView {
    pub code_text: String,
    pub name_text: String,
    pub segment_text: String,
    pub minimum: String,
    pub burst_text: String,
    pub grace_days_text: String,
    pub rotation_days: String,
    pub external_ref: String,
    pub starts_at_text: String,
    pub created_at: String,
    pub summary_text: String,
    pub status_label: String,
}

pub fn to_escalation_template_view(item: &EscalationTemplate) -> EscalationTemplateView {
    EscalationTemplateView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        segment_text: text::truncate(&item.segment, 40),
        minimum: format!("{:.2}", item.minimum),
        burst_text: item.burst.to_string(),
        grace_days_text: item.grace_days.to_string(),
        rotation_days: item.rotation_days.to_string(),
        external_ref: text::truncate(&item.external_ref, 80),
        starts_at_text: item.starts_at.to_string(),
        created_at: item.created_at.to_string(),
        summary_text: text::truncate(&item.summary, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_template_views(items: &[EscalationTemplate]) -> Vec<EscalationTemplateView> {
    let mut views: Vec<EscalationTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_template_view(item));
    }
    views
}

pub fn sum_escalation_template_view(view: &EscalationTemplateView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_minimum + (parsed_burst as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_escalation_template("code-328", "name-401");
        let view = to_escalation_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_escalation_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
