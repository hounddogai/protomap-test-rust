use super::*;

/// How screens and exports show a lenient probe milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientProbeMilestoneView {
    pub code_text: String,
    pub name: String,
    pub retries: String,
    pub burst_text: String,
    pub approved_text: String,
    pub ceiling: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_lenient_probe_milestone_view(item: &LenientProbeMilestone) -> LenientProbeMilestoneView {
    LenientProbeMilestoneView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        retries: item.retries.to_string(),
        burst_text: item.burst.to_string(),
        approved_text: format!("{}", item.approved),
        ceiling: format!("{:.2}", item.ceiling),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_probe_milestone_views(items: &[LenientProbeMilestone]) -> Vec<LenientProbeMilestoneView> {
    let mut views: Vec<LenientProbeMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_probe_milestone_view(item));
    }
    views
}

pub fn sum_lenient_probe_milestone_view(view: &LenientProbeMilestoneView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_burst as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_probe_milestone("code-357", "name-464");
        let view = to_lenient_probe_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_probe_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
