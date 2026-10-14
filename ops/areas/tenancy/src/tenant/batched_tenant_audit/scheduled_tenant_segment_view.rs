use super::*;

/// How screens and exports show a scheduled tenant segment.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantSegmentView {
    pub code_text: String,
    pub title_text: String,
    pub segment: String,
    pub cost: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_scheduled_tenant_segment_view(item: &ScheduledTenantSegment) -> ScheduledTenantSegmentView {
    ScheduledTenantSegmentView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        segment: item.segment.clone(),
        cost: format!("{:.2}", item.cost),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_tenant_segment_views(items: &[ScheduledTenantSegment]) -> Vec<ScheduledTenantSegmentView> {
    let mut views: Vec<ScheduledTenantSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_tenant_segment_view(item));
    }
    views
}

pub fn sum_scheduled_tenant_segment_view(view: &ScheduledTenantSegmentView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_cost + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_tenant_segment("code-660", "title-743");
        let view = to_scheduled_tenant_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_tenant_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
