use super::*;

/// How screens and exports show a overage batch.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageBatchView {
    pub code_text: String,
    pub title: String,
    pub quantity_text: String,
    pub segment_text: String,
    pub pinned: String,
    pub window_days_text: String,
    pub phone_text: String,
    pub status_label: String,
}

pub fn to_overage_batch_view(item: &OverageBatch) -> OverageBatchView {
    OverageBatchView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        quantity_text: item.quantity.to_string(),
        segment_text: item.segment.clone(),
        pinned: format!("{}", item.pinned),
        window_days_text: item.window_days.to_string(),
        phone_text: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_batch_views(items: &[OverageBatch]) -> Vec<OverageBatchView> {
    let mut views: Vec<OverageBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_batch_view(item));
    }
    views
}

pub fn sum_overage_batch_view(view: &OverageBatchView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_batch("code-606", "title-944");
        let view = to_overage_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
