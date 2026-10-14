use super::*;

use acme_kit::text;

/// How screens and exports show a manual failover cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFailoverCycleView {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub ceiling: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_manual_failover_cycle_view(item: &ManualFailoverCycle) -> ManualFailoverCycleView {
    ManualFailoverCycleView {
        code: item.code.clone(),
        title: item.title.clone(),
        namespace: text::truncate(&item.namespace, 120),
        ceiling: format!("{:.2}", item.ceiling),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_failover_cycle_views(items: &[ManualFailoverCycle]) -> Vec<ManualFailoverCycleView> {
    let mut views: Vec<ManualFailoverCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_failover_cycle_view(item));
    }
    views
}

pub fn sum_manual_failover_cycle_view(view: &ManualFailoverCycleView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_failover_cycle("code-506", "title-322");
        let view = to_manual_failover_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_failover_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
