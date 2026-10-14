use super::*;

/// How screens and exports show a timeline baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBaselineView {
    pub code: String,
    pub name: String,
    pub quantity: String,
    pub region: String,
    pub retries: String,
    pub ceiling_text: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_timeline_baseline_view(item: &TimelineBaseline) -> TimelineBaselineView {
    TimelineBaselineView {
        code: item.code.clone(),
        name: item.name.clone(),
        quantity: item.quantity.to_string(),
        region: item.region.clone(),
        retries: item.retries.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_baseline_views(items: &[TimelineBaseline]) -> Vec<TimelineBaselineView> {
    let mut views: Vec<TimelineBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_baseline_view(item));
    }
    views
}

pub fn sum_timeline_baseline_view(view: &TimelineBaselineView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_retries as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_baseline("code-351", "name-310");
        let view = to_timeline_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
