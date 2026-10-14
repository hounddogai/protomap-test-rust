use super::*;

/// How screens and exports show a strict grant segment.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictGrantSegmentView {
    pub code: String,
    pub title: String,
    pub amount_text: String,
    pub category: String,
    pub rotation_days: String,
    pub starts_at: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_strict_grant_segment_view(item: &StrictGrantSegment) -> StrictGrantSegmentView {
    StrictGrantSegmentView {
        code: item.code.clone(),
        title: item.title.clone(),
        amount_text: format!("{:.2}", item.amount),
        category: item.category.clone(),
        rotation_days: item.rotation_days.to_string(),
        starts_at: item.starts_at.to_string(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_grant_segment_views(items: &[StrictGrantSegment]) -> Vec<StrictGrantSegmentView> {
    let mut views: Vec<StrictGrantSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_grant_segment_view(item));
    }
    views
}

pub fn sum_strict_grant_segment_view(view: &StrictGrantSegmentView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_amount + (parsed_rotation_days as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_grant_segment("code-183", "title-707");
        let view = to_strict_grant_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_grant_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
