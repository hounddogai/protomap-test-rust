use super::*;

use acme_kit::text;

/// How screens and exports show a federation target.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationTargetView {
    pub code: String,
    pub name_text: String,
    pub scope: String,
    pub priority_text: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_federation_target_view(item: &FederationTarget) -> FederationTargetView {
    FederationTargetView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        scope: text::truncate(&item.scope, 40),
        priority_text: item.priority.to_string(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_target_views(items: &[FederationTarget]) -> Vec<FederationTargetView> {
    let mut views: Vec<FederationTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_target_view(item));
    }
    views
}

pub fn sum_federation_target_view(view: &FederationTargetView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_target("code-296", "name-405");
        let view = to_federation_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
