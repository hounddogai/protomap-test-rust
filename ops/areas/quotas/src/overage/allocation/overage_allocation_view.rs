use super::*;

/// How screens and exports show a overage allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageAllocationView {
    pub code: String,
    pub label: String,
    pub batch_size: String,
    pub window_days: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_overage_allocation_view(item: &OverageAllocation) -> OverageAllocationView {
    OverageAllocationView {
        code: item.code.clone(),
        label: item.label.clone(),
        batch_size: item.batch_size.to_string(),
        window_days: item.window_days.to_string(),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_allocation_views(items: &[OverageAllocation]) -> Vec<OverageAllocationView> {
    let mut views: Vec<OverageAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_allocation_view(item));
    }
    views
}

pub fn sum_overage_allocation_view(view: &OverageAllocationView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_allocation("code-771", "label-991");
        let view = to_overage_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
