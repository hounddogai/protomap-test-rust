use super::*;

use acme_kit::text;

/// How screens and exports show a global overage snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalOverageSnapshotView {
    pub code: String,
    pub name: String,
    pub budget_text: String,
    pub timezone: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_global_overage_snapshot_view(item: &GlobalOverageSnapshot) -> GlobalOverageSnapshotView {
    GlobalOverageSnapshotView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        budget_text: format!("{:.2}", item.budget),
        timezone: item.timezone.clone(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_overage_snapshot_views(items: &[GlobalOverageSnapshot]) -> Vec<GlobalOverageSnapshotView> {
    let mut views: Vec<GlobalOverageSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_overage_snapshot_view(item));
    }
    views
}

pub fn sum_global_overage_snapshot_view(view: &GlobalOverageSnapshotView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_budget + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_overage_snapshot("code-47", "name-831");
        let view = to_global_overage_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_overage_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
