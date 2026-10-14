use super::*;

use acme_kit::text;

/// How screens and exports show a identity checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityCheckpointView {
    pub code: String,
    pub name: String,
    pub rate: String,
    pub grace_days: String,
    pub description: String,
    pub principal_text: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_identity_checkpoint_view(item: &IdentityCheckpoint) -> IdentityCheckpointView {
    IdentityCheckpointView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        rate: format!("{:.2}", item.rate),
        grace_days: item.grace_days.to_string(),
        description: text::truncate(&item.description, 120),
        principal_text: item.principal.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_checkpoint_views(items: &[IdentityCheckpoint]) -> Vec<IdentityCheckpointView> {
    let mut views: Vec<IdentityCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_checkpoint_view(item));
    }
    views
}

pub fn sum_identity_checkpoint_view(view: &IdentityCheckpointView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_rate + (parsed_grace_days as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_checkpoint("code-134", "name-214");
        let view = to_identity_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
