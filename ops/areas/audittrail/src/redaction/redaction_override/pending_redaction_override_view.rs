use super::*;

use acme_kit::text;

/// How screens and exports show a pending redaction override.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionOverrideView {
    pub code: String,
    pub label: String,
    pub ceiling_text: String,
    pub port_text: String,
    pub discount_text: String,
    pub description: String,
    pub host: String,
    pub threshold_text: String,
    pub reviewer: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_pending_redaction_override_view(item: &PendingRedactionOverride) -> PendingRedactionOverrideView {
    PendingRedactionOverrideView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        ceiling_text: format!("{:.2}", item.ceiling),
        port_text: item.port.to_string(),
        discount_text: format!("{:.2}", item.discount),
        description: text::truncate(&item.description, 64),
        host: item.host.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        reviewer: item.reviewer.clone(),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_redaction_override_views(items: &[PendingRedactionOverride]) -> Vec<PendingRedactionOverrideView> {
    let mut views: Vec<PendingRedactionOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_redaction_override_view(item));
    }
    views
}

pub fn sum_pending_redaction_override_view(view: &PendingRedactionOverrideView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_ceiling + (parsed_port as f64) + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_redaction_override("code-749", "label-236");
        let view = to_pending_redaction_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_redaction_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
