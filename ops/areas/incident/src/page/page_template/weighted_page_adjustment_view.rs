use super::*;

/// How screens and exports show a weighted page adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPageAdjustmentView {
    pub code: String,
    pub label: String,
    pub rotation_days: String,
    pub limit: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_weighted_page_adjustment_view(item: &WeightedPageAdjustment) -> WeightedPageAdjustmentView {
    WeightedPageAdjustmentView {
        code: item.code.clone(),
        label: item.label.clone(),
        rotation_days: item.rotation_days.to_string(),
        limit: item.limit.to_string(),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_page_adjustment_views(items: &[WeightedPageAdjustment]) -> Vec<WeightedPageAdjustmentView> {
    let mut views: Vec<WeightedPageAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_page_adjustment_view(item));
    }
    views
}

pub fn sum_weighted_page_adjustment_view(view: &WeightedPageAdjustmentView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_page_adjustment("code-234", "label-285");
        let view = to_weighted_page_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_page_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
