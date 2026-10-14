use super::*;

use acme_kit::text;

/// How screens and exports show a weighted flow log mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFlowLogMappingView {
    pub code: String,
    pub name_text: String,
    pub title_text: String,
    pub cipher: String,
    pub approved_text: String,
    pub audience: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_weighted_flow_log_mapping_view(item: &WeightedFlowLogMapping) -> WeightedFlowLogMappingView {
    WeightedFlowLogMappingView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        title_text: item.title.clone(),
        cipher: item.cipher.clone(),
        approved_text: format!("{}", item.approved),
        audience: text::truncate(&item.audience, 64),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_flow_log_mapping_views(items: &[WeightedFlowLogMapping]) -> Vec<WeightedFlowLogMappingView> {
    let mut views: Vec<WeightedFlowLogMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_flow_log_mapping_view(item));
    }
    views
}

pub fn sum_weighted_flow_log_mapping_view(view: &WeightedFlowLogMappingView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_flow_log_mapping("code-609", "name-730");
        let view = to_weighted_flow_log_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_flow_log_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
