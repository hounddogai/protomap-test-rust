use super::*;

use acme_kit::text;

/// How screens and exports show a escalation snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationSnapshotView {
    pub code: String,
    pub label_text: String,
    pub quantity_text: String,
    pub contact_email: String,
    pub currency_text: String,
    pub retries_text: String,
    pub attempts_text: String,
    pub channel: String,
    pub name: String,
    pub status_label: String,
}

pub fn to_escalation_snapshot_view(item: &EscalationSnapshot) -> EscalationSnapshotView {
    EscalationSnapshotView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        quantity_text: item.quantity.to_string(),
        contact_email: text::truncate(&item.contact_email, 120),
        currency_text: item.currency.clone(),
        retries_text: item.retries.to_string(),
        attempts_text: item.attempts.to_string(),
        channel: item.channel.clone(),
        name: item.name.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_snapshot_views(items: &[EscalationSnapshot]) -> Vec<EscalationSnapshotView> {
    let mut views: Vec<EscalationSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_snapshot_view(item));
    }
    views
}

pub fn sum_escalation_snapshot_view(view: &EscalationSnapshotView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_retries as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_escalation_snapshot("code-220", "label-609");
        let view = to_escalation_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_escalation_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
