use super::*;

/// How screens and exports show a pending probe assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeAssignmentView {
    pub code: String,
    pub name: String,
    pub burst: String,
    pub cost: String,
    pub issuer_text: String,
    pub rate: String,
    pub approved: String,
    pub created_at: String,
    pub title_text: String,
    pub archived_text: String,
    pub status_label: String,
}

pub fn to_pending_probe_assignment_view(item: &PendingProbeAssignment) -> PendingProbeAssignmentView {
    PendingProbeAssignmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        burst: item.burst.to_string(),
        cost: format!("{:.2}", item.cost),
        issuer_text: item.issuer.clone(),
        rate: format!("{:.2}", item.rate),
        approved: format!("{}", item.approved),
        created_at: item.created_at.to_string(),
        title_text: item.title.clone(),
        archived_text: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_probe_assignment_views(items: &[PendingProbeAssignment]) -> Vec<PendingProbeAssignmentView> {
    let mut views: Vec<PendingProbeAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_probe_assignment_view(item));
    }
    views
}

pub fn sum_pending_probe_assignment_view(view: &PendingProbeAssignmentView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_burst as f64 + parsed_cost + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_probe_assignment("code-605", "name-69");
        let view = to_pending_probe_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_probe_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
