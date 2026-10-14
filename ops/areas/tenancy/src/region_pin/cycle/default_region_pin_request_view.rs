use super::*;

use acme_kit::text;

/// How screens and exports show a default region pin request.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRegionPinRequestView {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub phone: String,
    pub rate_text: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_default_region_pin_request_view(item: &DefaultRegionPinRequest) -> DefaultRegionPinRequestView {
    DefaultRegionPinRequestView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        timezone: item.timezone.clone(),
        phone: item.phone.clone(),
        rate_text: format!("{:.2}", item.rate),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_region_pin_request_views(items: &[DefaultRegionPinRequest]) -> Vec<DefaultRegionPinRequestView> {
    let mut views: Vec<DefaultRegionPinRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_region_pin_request_view(item));
    }
    views
}

pub fn sum_default_region_pin_request_view(view: &DefaultRegionPinRequestView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_region_pin_request("code-993", "name-548");
        let view = to_default_region_pin_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_region_pin_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
