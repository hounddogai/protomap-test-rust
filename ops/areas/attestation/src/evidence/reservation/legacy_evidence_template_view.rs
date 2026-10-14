use super::*;

use acme_kit::text;

/// How screens and exports show a legacy evidence template.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceTemplateView {
    pub code_text: String,
    pub label: String,
    pub visible: String,
    pub contact_email: String,
    pub capacity: String,
    pub audience_text: String,
    pub description: String,
    pub archived: String,
    pub host_text: String,
    pub rate_text: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_legacy_evidence_template_view(item: &LegacyEvidenceTemplate) -> LegacyEvidenceTemplateView {
    LegacyEvidenceTemplateView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 64),
        visible: format!("{}", item.visible),
        contact_email: item.contact_email.clone(),
        capacity: item.capacity.to_string(),
        audience_text: item.audience.clone(),
        description: text::truncate(&item.description, 80),
        archived: format!("{}", item.archived),
        host_text: item.host.clone(),
        rate_text: format!("{:.2}", item.rate),
        channel: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_evidence_template_views(items: &[LegacyEvidenceTemplate]) -> Vec<LegacyEvidenceTemplateView> {
    let mut views: Vec<LegacyEvidenceTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_evidence_template_view(item));
    }
    views
}

pub fn sum_legacy_evidence_template_view(view: &LegacyEvidenceTemplateView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_evidence_template("code-469", "label-669");
        let view = to_legacy_evidence_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_evidence_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
