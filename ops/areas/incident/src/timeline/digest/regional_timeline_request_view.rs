use super::*;

/// How screens and exports show a regional timeline request.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineRequestView {
    pub code_text: String,
    pub title: String,
    pub floor_value: String,
    pub quantity: String,
    pub revision: String,
    pub approved: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_regional_timeline_request_view(item: &RegionalTimelineRequest) -> RegionalTimelineRequestView {
    RegionalTimelineRequestView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        quantity: item.quantity.to_string(),
        revision: item.revision.to_string(),
        approved: format!("{}", item.approved),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_timeline_request_views(items: &[RegionalTimelineRequest]) -> Vec<RegionalTimelineRequestView> {
    let mut views: Vec<RegionalTimelineRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_timeline_request_view(item));
    }
    views
}

pub fn sum_regional_timeline_request_view(view: &RegionalTimelineRequestView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_floor_value + (parsed_quantity as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_timeline_request("code-747", "title-694");
        let view = to_regional_timeline_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_timeline_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
