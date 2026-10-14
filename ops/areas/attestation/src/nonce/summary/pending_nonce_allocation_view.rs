use super::*;

/// How screens and exports show a pending nonce allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceAllocationView {
    pub code: String,
    pub title: String,
    pub version_text: String,
    pub sequence: String,
    pub phone_text: String,
    pub source: String,
    pub namespace_text: String,
    pub capacity: String,
    pub amount: String,
    pub port: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_pending_nonce_allocation_view(item: &PendingNonceAllocation) -> PendingNonceAllocationView {
    PendingNonceAllocationView {
        code: item.code.clone(),
        title: item.title.clone(),
        version_text: item.version.to_string(),
        sequence: item.sequence.to_string(),
        phone_text: item.phone.clone(),
        source: item.source.clone(),
        namespace_text: item.namespace.clone(),
        capacity: item.capacity.to_string(),
        amount: format!("{:.2}", item.amount),
        port: item.port.to_string(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_nonce_allocation_views(items: &[PendingNonceAllocation]) -> Vec<PendingNonceAllocationView> {
    let mut views: Vec<PendingNonceAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_nonce_allocation_view(item));
    }
    views
}

pub fn sum_pending_nonce_allocation_view(view: &PendingNonceAllocationView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_version as f64 + (parsed_sequence as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_nonce_allocation("code-678", "title-345");
        let view = to_pending_nonce_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_nonce_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
