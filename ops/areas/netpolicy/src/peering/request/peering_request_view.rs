use super::*;

use acme_kit::text;

/// How screens and exports show a peering request.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRequestView {
    pub code: String,
    pub name: String,
    pub created_at: String,
    pub scope: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_peering_request_view(item: &PeeringRequest) -> PeeringRequestView {
    PeeringRequestView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        created_at: item.created_at.to_string(),
        scope: text::truncate(&item.scope, 80),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_request_views(items: &[PeeringRequest]) -> Vec<PeeringRequestView> {
    let mut views: Vec<PeeringRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_request_view(item));
    }
    views
}

pub fn sum_peering_request_view(view: &PeeringRequestView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_created_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_request("code-753", "name-198");
        let view = to_peering_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
