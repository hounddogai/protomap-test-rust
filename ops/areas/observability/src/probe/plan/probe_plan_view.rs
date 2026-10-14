use super::*;

use acme_kit::text;

/// How screens and exports show a probe plan.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbePlanView {
    pub code: String,
    pub label_text: String,
    pub notes: String,
    pub segment: String,
    pub minimum: String,
    pub version: String,
    pub summary: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_probe_plan_view(item: &ProbePlan) -> ProbePlanView {
    ProbePlanView {
        code: text::truncate(&item.code, 80),
        label_text: text::truncate(&item.label, 40),
        notes: text::truncate(&item.notes, 64),
        segment: item.segment.clone(),
        minimum: format!("{:.2}", item.minimum),
        version: item.version.to_string(),
        summary: item.summary.clone(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_plan_views(items: &[ProbePlan]) -> Vec<ProbePlanView> {
    let mut views: Vec<ProbePlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_plan_view(item));
    }
    views
}

pub fn sum_probe_plan_view(view: &ProbePlanView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_minimum + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_plan("code-151", "label-351");
        let view = to_probe_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
