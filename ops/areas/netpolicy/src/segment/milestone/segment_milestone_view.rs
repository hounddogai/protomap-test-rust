use super::*;

use acme_kit::text;

/// How screens and exports show a segment milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMilestoneView {
    pub code: String,
    pub label_text: String,
    pub reviewer: String,
    pub visible: String,
    pub ratio: String,
    pub revision: String,
    pub issuer_text: String,
    pub limit: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_segment_milestone_view(item: &SegmentMilestone) -> SegmentMilestoneView {
    SegmentMilestoneView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        reviewer: item.reviewer.clone(),
        visible: format!("{}", item.visible),
        ratio: format!("{:.2}", item.ratio),
        revision: item.revision.to_string(),
        issuer_text: item.issuer.clone(),
        limit: item.limit.to_string(),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_milestone_views(items: &[SegmentMilestone]) -> Vec<SegmentMilestoneView> {
    let mut views: Vec<SegmentMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_milestone_view(item));
    }
    views
}

pub fn sum_segment_milestone_view(view: &SegmentMilestoneView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_ratio + (parsed_revision as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_milestone("code-383", "label-556");
        let view = to_segment_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
