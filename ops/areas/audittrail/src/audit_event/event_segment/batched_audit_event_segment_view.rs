use super::*;

/// How screens and exports show a batched audit event segment.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventSegmentView {
    pub code_text: String,
    pub title: String,
    pub currency_text: String,
    pub key_id_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_batched_audit_event_segment_view(item: &BatchedAuditEventSegment) -> BatchedAuditEventSegmentView {
    BatchedAuditEventSegmentView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        currency_text: item.currency.clone(),
        key_id_text: item.key_id.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_audit_event_segment_views(items: &[BatchedAuditEventSegment]) -> Vec<BatchedAuditEventSegmentView> {
    let mut views: Vec<BatchedAuditEventSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_audit_event_segment_view(item));
    }
    views
}

pub fn sum_batched_audit_event_segment_view(view: &BatchedAuditEventSegmentView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_audit_event_segment("code-925", "title-973");
        let view = to_batched_audit_event_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_audit_event_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
