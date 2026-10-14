use super::*;

/// How screens and exports show a region pin checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinCheckpointView {
    pub code_text: String,
    pub title: String,
    pub limit: String,
    pub reviewer: String,
    pub ceiling: String,
    pub budget: String,
    pub archived_text: String,
    pub discount: String,
    pub category: String,
    pub burst: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_region_pin_checkpoint_view(item: &RegionPinCheckpoint) -> RegionPinCheckpointView {
    RegionPinCheckpointView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        limit: item.limit.to_string(),
        reviewer: item.reviewer.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        budget: format!("{:.2}", item.budget),
        archived_text: format!("{}", item.archived),
        discount: format!("{:.2}", item.discount),
        category: item.category.clone(),
        burst: item.burst.to_string(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_checkpoint_views(items: &[RegionPinCheckpoint]) -> Vec<RegionPinCheckpointView> {
    let mut views: Vec<RegionPinCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_checkpoint_view(item));
    }
    views
}

pub fn sum_region_pin_checkpoint_view(view: &RegionPinCheckpointView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_limit as f64 + parsed_ceiling + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_checkpoint("code-323", "title-550");
        let view = to_region_pin_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
