use super::*;

use acme_kit::text;

/// How screens and exports show a flow log profile.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogProfileView {
    pub code: String,
    pub name: String,
    pub pinned: String,
    pub rotation_days_text: String,
    pub sequence: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_flow_log_profile_view(item: &FlowLogProfile) -> FlowLogProfileView {
    FlowLogProfileView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        pinned: format!("{}", item.pinned),
        rotation_days_text: item.rotation_days.to_string(),
        sequence: item.sequence.to_string(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_profile_views(items: &[FlowLogProfile]) -> Vec<FlowLogProfileView> {
    let mut views: Vec<FlowLogProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_profile_view(item));
    }
    views
}

pub fn sum_flow_log_profile_view(view: &FlowLogProfileView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_profile("code-947", "name-178");
        let view = to_flow_log_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
