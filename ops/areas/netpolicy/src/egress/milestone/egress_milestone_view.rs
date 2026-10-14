use super::*;

use acme_kit::text;

/// How screens and exports show a egress milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressMilestoneView {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub country_text: String,
    pub namespace: String,
    pub ratio: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_egress_milestone_view(item: &EgressMilestone) -> EgressMilestoneView {
    EgressMilestoneView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        reviewer: text::truncate(&item.reviewer, 64),
        country_text: text::truncate(&item.country, 80),
        namespace: item.namespace.clone(),
        ratio: format!("{:.2}", item.ratio),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_milestone_views(items: &[EgressMilestone]) -> Vec<EgressMilestoneView> {
    let mut views: Vec<EgressMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_milestone_view(item));
    }
    views
}

pub fn sum_egress_milestone_view(view: &EgressMilestoneView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_ratio + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_milestone("code-541", "title-219");
        let view = to_egress_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
