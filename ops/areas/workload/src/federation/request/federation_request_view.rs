use super::*;

/// How screens and exports show a federation request.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationRequestView {
    pub code_text: String,
    pub name_text: String,
    pub quota_units_text: String,
    pub title: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_federation_request_view(item: &FederationRequest) -> FederationRequestView {
    FederationRequestView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        quota_units_text: item.quota_units.to_string(),
        title: item.title.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_request_views(items: &[FederationRequest]) -> Vec<FederationRequestView> {
    let mut views: Vec<FederationRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_request_view(item));
    }
    views
}

pub fn sum_federation_request_view(view: &FederationRequestView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_request("code-318", "name-329");
        let view = to_federation_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
