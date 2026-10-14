use super::*;

use acme_kit::text;

/// How screens and exports show a regional approval rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalRollupView {
    pub code: String,
    pub label: String,
    pub owner_id_text: String,
    pub visible_text: String,
    pub name: String,
    pub weight: String,
    pub ttl_seconds: String,
    pub channel_text: String,
    pub notes: String,
    pub ceiling_text: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_regional_approval_rollup_view(item: &RegionalApprovalRollup) -> RegionalApprovalRollupView {
    RegionalApprovalRollupView {
        code: item.code.clone(),
        label: item.label.clone(),
        owner_id_text: text::truncate(&item.owner_id, 120),
        visible_text: format!("{}", item.visible),
        name: item.name.clone(),
        weight: format!("{:.2}", item.weight),
        ttl_seconds: item.ttl_seconds.to_string(),
        channel_text: item.channel.clone(),
        notes: text::truncate(&item.notes, 120),
        ceiling_text: format!("{:.2}", item.ceiling),
        timezone: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_approval_rollup_views(items: &[RegionalApprovalRollup]) -> Vec<RegionalApprovalRollupView> {
    let mut views: Vec<RegionalApprovalRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_approval_rollup_view(item));
    }
    views
}

pub fn sum_regional_approval_rollup_view(view: &RegionalApprovalRollupView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_weight + (parsed_ttl_seconds as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_approval_rollup("code-38", "label-891");
        let view = to_regional_approval_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_approval_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
