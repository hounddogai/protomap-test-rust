use super::*;

use acme_kit::text;

/// How screens and exports show a pending silence allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSilenceAllocationView {
    pub code: String,
    pub name: String,
    pub amount_text: String,
    pub starts_at: String,
    pub batch_size_text: String,
    pub limit_text: String,
    pub threshold: String,
    pub tenant_id: String,
    pub approved_text: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_pending_silence_allocation_view(item: &PendingSilenceAllocation) -> PendingSilenceAllocationView {
    PendingSilenceAllocationView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        amount_text: format!("{:.2}", item.amount),
        starts_at: item.starts_at.to_string(),
        batch_size_text: item.batch_size.to_string(),
        limit_text: item.limit.to_string(),
        threshold: format!("{:.2}", item.threshold),
        tenant_id: item.tenant_id.clone(),
        approved_text: format!("{}", item.approved),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_silence_allocation_views(items: &[PendingSilenceAllocation]) -> Vec<PendingSilenceAllocationView> {
    let mut views: Vec<PendingSilenceAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_silence_allocation_view(item));
    }
    views
}

pub fn sum_pending_silence_allocation_view(view: &PendingSilenceAllocationView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_amount + (parsed_starts_at as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_silence_allocation("code-247", "name-417");
        let view = to_pending_silence_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_silence_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
