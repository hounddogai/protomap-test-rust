use super::*;

use acme_kit::text;

/// How screens and exports show a forecast tier.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastTierView {
    pub code: String,
    pub name: String,
    pub country: String,
    pub contact_email: String,
    pub audience_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_forecast_tier_view(item: &ForecastTier) -> ForecastTierView {
    ForecastTierView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        country: text::truncate(&item.country, 80),
        contact_email: text::truncate(&item.contact_email, 120),
        audience_text: item.audience.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_tier_views(items: &[ForecastTier]) -> Vec<ForecastTierView> {
    let mut views: Vec<ForecastTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_tier_view(item));
    }
    views
}

pub fn sum_forecast_tier_view(view: &ForecastTierView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_tier("code-177", "name-392");
        let view = to_forecast_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
