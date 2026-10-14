use super::*;

use acme_kit::text;

/// How screens and exports show a regional usage segment.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageSegmentView {
    pub code: String,
    pub name_text: String,
    pub approved: String,
    pub audience: String,
    pub host_text: String,
    pub cipher_text: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_regional_usage_segment_view(item: &RegionalUsageSegment) -> RegionalUsageSegmentView {
    RegionalUsageSegmentView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        approved: format!("{}", item.approved),
        audience: item.audience.clone(),
        host_text: text::truncate(&item.host, 64),
        cipher_text: item.cipher.clone(),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_usage_segment_views(items: &[RegionalUsageSegment]) -> Vec<RegionalUsageSegmentView> {
    let mut views: Vec<RegionalUsageSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_usage_segment_view(item));
    }
    views
}

pub fn sum_regional_usage_segment_view(view: &RegionalUsageSegmentView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_usage_segment("code-100", "name-928");
        let view = to_regional_usage_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_usage_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
