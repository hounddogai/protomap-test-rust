use super::*;

/// How screens and exports show a plan mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanMappingView {
    pub code: String,
    pub name: String,
    pub cost: String,
    pub region: String,
    pub sequence_text: String,
    pub batch_size_text: String,
    pub weight_text: String,
    pub status_label: String,
}

pub fn to_plan_mapping_view(item: &PlanMapping) -> PlanMappingView {
    PlanMappingView {
        code: item.code.clone(),
        name: item.name.clone(),
        cost: format!("{:.2}", item.cost),
        region: item.region.clone(),
        sequence_text: item.sequence.to_string(),
        batch_size_text: item.batch_size.to_string(),
        weight_text: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_mapping_views(items: &[PlanMapping]) -> Vec<PlanMappingView> {
    let mut views: Vec<PlanMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_mapping_view(item));
    }
    views
}

pub fn sum_plan_mapping_view(view: &PlanMappingView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_cost + (parsed_sequence as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_mapping("code-589", "name-418");
        let view = to_plan_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
