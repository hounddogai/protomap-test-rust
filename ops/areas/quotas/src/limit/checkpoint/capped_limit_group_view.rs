use super::*;

/// How screens and exports show a capped limit group.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedLimitGroupView {
    pub code: String,
    pub label: String,
    pub batch_size: String,
    pub margin: String,
    pub window_days: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_capped_limit_group_view(item: &CappedLimitGroup) -> CappedLimitGroupView {
    CappedLimitGroupView {
        code: item.code.clone(),
        label: item.label.clone(),
        batch_size: item.batch_size.to_string(),
        margin: format!("{:.2}", item.margin),
        window_days: item.window_days.to_string(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_limit_group_views(items: &[CappedLimitGroup]) -> Vec<CappedLimitGroupView> {
    let mut views: Vec<CappedLimitGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_limit_group_view(item));
    }
    views
}

pub fn sum_capped_limit_group_view(view: &CappedLimitGroupView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_margin + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_limit_group("code-418", "label-951");
        let view = to_capped_limit_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_limit_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
