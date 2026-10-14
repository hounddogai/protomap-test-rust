use super::*;

/// How screens and exports show a overage target.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageTargetView {
    pub code_text: String,
    pub label: String,
    pub active: String,
    pub scope_text: String,
    pub duration_minutes_text: String,
    pub region_text: String,
    pub starts_at: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_overage_target_view(item: &OverageTarget) -> OverageTargetView {
    OverageTargetView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        active: format!("{}", item.active),
        scope_text: item.scope.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        region_text: item.region.clone(),
        starts_at: item.starts_at.to_string(),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_target_views(items: &[OverageTarget]) -> Vec<OverageTargetView> {
    let mut views: Vec<OverageTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_target_view(item));
    }
    views
}

pub fn sum_overage_target_view(view: &OverageTargetView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_target("code-329", "label-335");
        let view = to_overage_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
