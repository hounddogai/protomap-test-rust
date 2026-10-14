use super::*;

use acme_kit::text;

/// How screens and exports show a evidence template.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTemplateView {
    pub code: String,
    pub label: String,
    pub channel_text: String,
    pub owner_id_text: String,
    pub expires_at: String,
    pub phone: String,
    pub reviewer: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_evidence_template_view(item: &EvidenceTemplate) -> EvidenceTemplateView {
    EvidenceTemplateView {
        code: item.code.clone(),
        label: item.label.clone(),
        channel_text: item.channel.clone(),
        owner_id_text: item.owner_id.clone(),
        expires_at: item.expires_at.to_string(),
        phone: text::truncate(&item.phone, 40),
        reviewer: item.reviewer.clone(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_template_views(items: &[EvidenceTemplate]) -> Vec<EvidenceTemplateView> {
    let mut views: Vec<EvidenceTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_template_view(item));
    }
    views
}

pub fn sum_evidence_template_view(view: &EvidenceTemplateView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_template("code-638", "label-121");
        let view = to_evidence_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
