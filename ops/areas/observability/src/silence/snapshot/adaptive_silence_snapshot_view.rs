use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive silence snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceSnapshotView {
    pub code: String,
    pub name: String,
    pub description: String,
    pub issuer: String,
    pub budget: String,
    pub created_at: String,
    pub batch_size: String,
    pub namespace: String,
    pub approved: String,
    pub owner_id: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_adaptive_silence_snapshot_view(item: &AdaptiveSilenceSnapshot) -> AdaptiveSilenceSnapshotView {
    AdaptiveSilenceSnapshotView {
        code: item.code.clone(),
        name: item.name.clone(),
        description: item.description.clone(),
        issuer: item.issuer.clone(),
        budget: format!("{:.2}", item.budget),
        created_at: item.created_at.to_string(),
        batch_size: item.batch_size.to_string(),
        namespace: text::truncate(&item.namespace, 40),
        approved: format!("{}", item.approved),
        owner_id: item.owner_id.clone(),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_silence_snapshot_views(items: &[AdaptiveSilenceSnapshot]) -> Vec<AdaptiveSilenceSnapshotView> {
    let mut views: Vec<AdaptiveSilenceSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_silence_snapshot_view(item));
    }
    views
}

pub fn sum_adaptive_silence_snapshot_view(view: &AdaptiveSilenceSnapshotView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_budget + (parsed_created_at as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_silence_snapshot("code-615", "name-765");
        let view = to_adaptive_silence_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_silence_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
