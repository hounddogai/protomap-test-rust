use super::*;

/// How screens and exports show a adaptive query mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryMappingView {
    pub code: String,
    pub label: String,
    pub capacity_text: String,
    pub margin: String,
    pub maximum_text: String,
    pub status_label: String,
}

pub fn to_adaptive_query_mapping_view(item: &AdaptiveQueryMapping) -> AdaptiveQueryMappingView {
    AdaptiveQueryMappingView {
        code: item.code.clone(),
        label: item.label.clone(),
        capacity_text: item.capacity.to_string(),
        margin: format!("{:.2}", item.margin),
        maximum_text: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_query_mapping_views(items: &[AdaptiveQueryMapping]) -> Vec<AdaptiveQueryMappingView> {
    let mut views: Vec<AdaptiveQueryMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_query_mapping_view(item));
    }
    views
}

pub fn sum_adaptive_query_mapping_view(view: &AdaptiveQueryMappingView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_margin + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_query_mapping("code-113", "label-998");
        let view = to_adaptive_query_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_query_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
