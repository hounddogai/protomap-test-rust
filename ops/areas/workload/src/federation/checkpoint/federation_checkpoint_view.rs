use super::*;

use acme_kit::text;

/// How screens and exports show a federation checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationCheckpointView {
    pub code: String,
    pub name_text: String,
    pub version: String,
    pub currency: String,
    pub principal_text: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_federation_checkpoint_view(item: &FederationCheckpoint) -> FederationCheckpointView {
    FederationCheckpointView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        version: item.version.to_string(),
        currency: item.currency.clone(),
        principal_text: item.principal.clone(),
        tenant_id_text: text::truncate(&item.tenant_id, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_checkpoint_views(items: &[FederationCheckpoint]) -> Vec<FederationCheckpointView> {
    let mut views: Vec<FederationCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_checkpoint_view(item));
    }
    views
}

pub fn sum_federation_checkpoint_view(view: &FederationCheckpointView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_version as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_checkpoint("code-976", "name-89");
        let view = to_federation_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
