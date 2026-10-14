use super::*;

/// How screens and exports show a forecast allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAllocationView {
    pub code: String,
    pub name_text: String,
    pub region_text: String,
    pub updated_at_text: String,
    pub contact_email_text: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_forecast_allocation_view(item: &ForecastAllocation) -> ForecastAllocationView {
    ForecastAllocationView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        region_text: item.region.clone(),
        updated_at_text: item.updated_at.to_string(),
        contact_email_text: item.contact_email.clone(),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_allocation_views(items: &[ForecastAllocation]) -> Vec<ForecastAllocationView> {
    let mut views: Vec<ForecastAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_allocation_view(item));
    }
    views
}

pub fn sum_forecast_allocation_view(view: &ForecastAllocationView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_updated_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_allocation("code-978", "name-787");
        let view = to_forecast_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
