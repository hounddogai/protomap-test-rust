use super::*;

use acme_kit::text;

/// How screens and exports show a rollback baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackBaselineView {
    pub code_text: String,
    pub title: String,
    pub segment: String,
    pub sequence: String,
    pub summary: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_rollback_baseline_view(item: &RollbackBaseline) -> RollbackBaselineView {
    RollbackBaselineView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        segment: item.segment.clone(),
        sequence: item.sequence.to_string(),
        summary: text::truncate(&item.summary, 80),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_baseline_views(items: &[RollbackBaseline]) -> Vec<RollbackBaselineView> {
    let mut views: Vec<RollbackBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_baseline_view(item));
    }
    views
}

pub fn sum_rollback_baseline_view(view: &RollbackBaselineView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_baseline("code-964", "title-220");
        let view = to_rollback_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
