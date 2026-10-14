use super::*;

/// How screens and exports show a tiered rollout mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutMappingView {
    pub code_text: String,
    pub label_text: String,
    pub revision_text: String,
    pub scope_text: String,
    pub duration_minutes: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_tiered_rollout_mapping_view(item: &TieredRolloutMapping) -> TieredRolloutMappingView {
    TieredRolloutMappingView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        revision_text: item.revision.to_string(),
        scope_text: item.scope.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_rollout_mapping_views(items: &[TieredRolloutMapping]) -> Vec<TieredRolloutMappingView> {
    let mut views: Vec<TieredRolloutMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_rollout_mapping_view(item));
    }
    views
}

pub fn sum_tiered_rollout_mapping_view(view: &TieredRolloutMappingView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_duration_minutes as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_rollout_mapping("code-734", "label-881");
        let view = to_tiered_rollout_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_rollout_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
