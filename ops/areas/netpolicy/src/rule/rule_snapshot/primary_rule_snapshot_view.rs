use super::*;

/// How screens and exports show a primary rule snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleSnapshotView {
    pub code_text: String,
    pub title: String,
    pub archived_text: String,
    pub duration_minutes_text: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_primary_rule_snapshot_view(item: &PrimaryRuleSnapshot) -> PrimaryRuleSnapshotView {
    PrimaryRuleSnapshotView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        archived_text: format!("{}", item.archived),
        duration_minutes_text: item.duration_minutes.to_string(),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_rule_snapshot_views(items: &[PrimaryRuleSnapshot]) -> Vec<PrimaryRuleSnapshotView> {
    let mut views: Vec<PrimaryRuleSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_rule_snapshot_view(item));
    }
    views
}

pub fn sum_primary_rule_snapshot_view(view: &PrimaryRuleSnapshotView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_rule_snapshot("code-875", "title-349");
        let view = to_primary_rule_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_rule_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
