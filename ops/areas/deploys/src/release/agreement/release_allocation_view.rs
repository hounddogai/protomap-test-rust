use super::*;

/// How screens and exports show a release allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAllocationView {
    pub code: String,
    pub title: String,
    pub weight: String,
    pub updated_at: String,
    pub visible_text: String,
    pub rotation_days: String,
    pub cluster: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_release_allocation_view(item: &ReleaseAllocation) -> ReleaseAllocationView {
    ReleaseAllocationView {
        code: item.code.clone(),
        title: item.title.clone(),
        weight: format!("{:.2}", item.weight),
        updated_at: item.updated_at.to_string(),
        visible_text: format!("{}", item.visible),
        rotation_days: item.rotation_days.to_string(),
        cluster: item.cluster.clone(),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_allocation_views(items: &[ReleaseAllocation]) -> Vec<ReleaseAllocationView> {
    let mut views: Vec<ReleaseAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_allocation_view(item));
    }
    views
}

pub fn sum_release_allocation_view(view: &ReleaseAllocationView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_weight + (parsed_updated_at as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_allocation("code-773", "title-119");
        let view = to_release_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
