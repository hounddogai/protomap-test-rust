use super::*;

/// How screens and exports show a peering group.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringGroupView {
    pub code_text: String,
    pub title: String,
    pub sequence_text: String,
    pub priority: String,
    pub updated_at: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_peering_group_view(item: &PeeringGroup) -> PeeringGroupView {
    PeeringGroupView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        sequence_text: item.sequence.to_string(),
        priority: item.priority.to_string(),
        updated_at: item.updated_at.to_string(),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_group_views(items: &[PeeringGroup]) -> Vec<PeeringGroupView> {
    let mut views: Vec<PeeringGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_group_view(item));
    }
    views
}

pub fn sum_peering_group_view(view: &PeeringGroupView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_priority as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_group("code-282", "title-481");
        let view = to_peering_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
