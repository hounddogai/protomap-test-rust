use super::*;

use acme_kit::text;

/// How screens and exports show a rollback cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCycleView {
    pub code_text: String,
    pub name_text: String,
    pub cluster: String,
    pub title: String,
    pub archived: String,
    pub approved_text: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_rollback_cycle_view(item: &RollbackCycle) -> RollbackCycleView {
    RollbackCycleView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        cluster: text::truncate(&item.cluster, 80),
        title: item.title.clone(),
        archived: format!("{}", item.archived),
        approved_text: format!("{}", item.approved),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_cycle_views(items: &[RollbackCycle]) -> Vec<RollbackCycleView> {
    let mut views: Vec<RollbackCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_cycle_view(item));
    }
    views
}

pub fn sum_rollback_cycle_view(view: &RollbackCycleView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_capacity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_cycle("code-900", "name-28");
        let view = to_rollback_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
