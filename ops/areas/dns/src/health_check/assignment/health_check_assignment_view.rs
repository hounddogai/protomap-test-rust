use super::*;

/// How screens and exports show a health check assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAssignmentView {
    pub code: String,
    pub label_text: String,
    pub checksum: String,
    pub visible: String,
    pub title: String,
    pub limit: String,
    pub retries: String,
    pub ceiling: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_health_check_assignment_view(item: &HealthCheckAssignment) -> HealthCheckAssignmentView {
    HealthCheckAssignmentView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        checksum: item.checksum.clone(),
        visible: format!("{}", item.visible),
        title: item.title.clone(),
        limit: item.limit.to_string(),
        retries: item.retries.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_assignment_views(items: &[HealthCheckAssignment]) -> Vec<HealthCheckAssignmentView> {
    let mut views: Vec<HealthCheckAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_assignment_view(item));
    }
    views
}

pub fn sum_health_check_assignment_view(view: &HealthCheckAssignmentView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_retries as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_assignment("code-229", "label-274");
        let view = to_health_check_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
