use super::*;

use acme_kit::text;

/// How screens and exports show a fallback binding milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingMilestoneView {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub updated_at_text: String,
    pub fingerprint: String,
    pub rate_text: String,
    pub approved_text: String,
    pub contact_email_text: String,
    pub segment: String,
    pub timezone: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_fallback_binding_milestone_view(item: &FallbackBindingMilestone) -> FallbackBindingMilestoneView {
    FallbackBindingMilestoneView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        notes: text::truncate(&item.notes, 40),
        updated_at_text: item.updated_at.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 120),
        rate_text: format!("{:.2}", item.rate),
        approved_text: format!("{}", item.approved),
        contact_email_text: item.contact_email.clone(),
        segment: item.segment.clone(),
        timezone: item.timezone.clone(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_binding_milestone_views(items: &[FallbackBindingMilestone]) -> Vec<FallbackBindingMilestoneView> {
    let mut views: Vec<FallbackBindingMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_binding_milestone_view(item));
    }
    views
}

pub fn sum_fallback_binding_milestone_view(view: &FallbackBindingMilestoneView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_rate + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_binding_milestone("code-450", "label-439");
        let view = to_fallback_binding_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_binding_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
