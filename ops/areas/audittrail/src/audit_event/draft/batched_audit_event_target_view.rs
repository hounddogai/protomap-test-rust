use super::*;

use acme_kit::text;

/// How screens and exports show a batched audit event target.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventTargetView {
    pub code_text: String,
    pub name: String,
    pub rate: String,
    pub cipher_text: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_batched_audit_event_target_view(item: &BatchedAuditEventTarget) -> BatchedAuditEventTargetView {
    BatchedAuditEventTargetView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 80),
        rate: format!("{:.2}", item.rate),
        cipher_text: text::truncate(&item.cipher, 80),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_audit_event_target_views(items: &[BatchedAuditEventTarget]) -> Vec<BatchedAuditEventTargetView> {
    let mut views: Vec<BatchedAuditEventTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_audit_event_target_view(item));
    }
    views
}

pub fn sum_batched_audit_event_target_view(view: &BatchedAuditEventTargetView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_rate + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_audit_event_target("code-253", "name-62");
        let view = to_batched_audit_event_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_audit_event_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
