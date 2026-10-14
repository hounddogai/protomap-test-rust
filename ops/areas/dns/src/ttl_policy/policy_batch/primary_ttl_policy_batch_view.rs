use super::*;

use acme_kit::text;

/// How screens and exports show a primary ttl policy batch.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyBatchView {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub name: String,
    pub capacity: String,
    pub archived: String,
    pub sequence: String,
    pub summary_text: String,
    pub unit: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_primary_ttl_policy_batch_view(item: &PrimaryTtlPolicyBatch) -> PrimaryTtlPolicyBatchView {
    PrimaryTtlPolicyBatchView {
        code: item.code.clone(),
        title: item.title.clone(),
        cluster: text::truncate(&item.cluster, 80),
        name: item.name.clone(),
        capacity: item.capacity.to_string(),
        archived: format!("{}", item.archived),
        sequence: item.sequence.to_string(),
        summary_text: text::truncate(&item.summary, 40),
        unit: item.unit.clone(),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_ttl_policy_batch_views(items: &[PrimaryTtlPolicyBatch]) -> Vec<PrimaryTtlPolicyBatchView> {
    let mut views: Vec<PrimaryTtlPolicyBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_ttl_policy_batch_view(item));
    }
    views
}

pub fn sum_primary_ttl_policy_batch_view(view: &PrimaryTtlPolicyBatchView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_ttl_policy_batch("code-536", "title-276");
        let view = to_primary_ttl_policy_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_ttl_policy_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
