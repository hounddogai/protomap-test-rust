use super::*;

/// How screens and exports show a plan batch.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanBatchView {
    pub code: String,
    pub title_text: String,
    pub version_text: String,
    pub capacity_text: String,
    pub starts_at: String,
    pub pinned_text: String,
    pub status_label: String,
}

pub fn to_plan_batch_view(item: &PlanBatch) -> PlanBatchView {
    PlanBatchView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        version_text: item.version.to_string(),
        capacity_text: item.capacity.to_string(),
        starts_at: item.starts_at.to_string(),
        pinned_text: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_batch_views(items: &[PlanBatch]) -> Vec<PlanBatchView> {
    let mut views: Vec<PlanBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_batch_view(item));
    }
    views
}

pub fn sum_plan_batch_view(view: &PlanBatchView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_version as f64 + (parsed_capacity as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_batch("code-198", "title-776");
        let view = to_plan_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
