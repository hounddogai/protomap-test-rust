use super::*;

use acme_kit::text;

/// How screens and exports show a manual rule snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleSnapshotView {
    pub code: String,
    pub name_text: String,
    pub summary_text: String,
    pub rotation_days: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_manual_rule_snapshot_view(item: &ManualRuleSnapshot) -> ManualRuleSnapshotView {
    ManualRuleSnapshotView {
        code: text::truncate(&item.code, 80),
        name_text: text::truncate(&item.name, 40),
        summary_text: text::truncate(&item.summary, 40),
        rotation_days: item.rotation_days.to_string(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_rule_snapshot_views(items: &[ManualRuleSnapshot]) -> Vec<ManualRuleSnapshotView> {
    let mut views: Vec<ManualRuleSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_rule_snapshot_view(item));
    }
    views
}

pub fn sum_manual_rule_snapshot_view(view: &ManualRuleSnapshotView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_rule_snapshot("code-104", "name-104");
        let view = to_manual_rule_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_rule_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
