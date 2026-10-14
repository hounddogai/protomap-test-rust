use super::*;

use acme_kit::text;

/// How screens and exports show a capped probe override.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeOverrideView {
    pub code_text: String,
    pub name: String,
    pub updated_at: String,
    pub region: String,
    pub scope: String,
    pub sequence: String,
    pub owner_id_text: String,
    pub label: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_capped_probe_override_view(item: &CappedProbeOverride) -> CappedProbeOverrideView {
    CappedProbeOverrideView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        updated_at: item.updated_at.to_string(),
        region: item.region.clone(),
        scope: item.scope.clone(),
        sequence: item.sequence.to_string(),
        owner_id_text: item.owner_id.clone(),
        label: text::truncate(&item.label, 64),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_probe_override_views(items: &[CappedProbeOverride]) -> Vec<CappedProbeOverrideView> {
    let mut views: Vec<CappedProbeOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_probe_override_view(item));
    }
    views
}

pub fn sum_capped_probe_override_view(view: &CappedProbeOverrideView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_sequence as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_probe_override("code-250", "name-763");
        let view = to_capped_probe_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_probe_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
