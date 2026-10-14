use super::*;

use acme_kit::text;

/// How screens and exports show a delegation checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationCheckpointView {
    pub code_text: String,
    pub name: String,
    pub fingerprint: String,
    pub cluster_text: String,
    pub ttl_seconds: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_delegation_checkpoint_view(item: &DelegationCheckpoint) -> DelegationCheckpointView {
    DelegationCheckpointView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        fingerprint: item.fingerprint.clone(),
        cluster_text: text::truncate(&item.cluster, 40),
        ttl_seconds: item.ttl_seconds.to_string(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_checkpoint_views(items: &[DelegationCheckpoint]) -> Vec<DelegationCheckpointView> {
    let mut views: Vec<DelegationCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_checkpoint_view(item));
    }
    views
}

pub fn sum_delegation_checkpoint_view(view: &DelegationCheckpointView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_checkpoint("code-572", "name-561");
        let view = to_delegation_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
