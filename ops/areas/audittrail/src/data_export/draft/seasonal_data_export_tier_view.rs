use super::*;

/// How screens and exports show a seasonal data export tier.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportTierView {
    pub code: String,
    pub title: String,
    pub margin_text: String,
    pub grace_days_text: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_seasonal_data_export_tier_view(item: &SeasonalDataExportTier) -> SeasonalDataExportTierView {
    SeasonalDataExportTierView {
        code: item.code.clone(),
        title: item.title.clone(),
        margin_text: format!("{:.2}", item.margin),
        grace_days_text: item.grace_days.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_data_export_tier_views(items: &[SeasonalDataExportTier]) -> Vec<SeasonalDataExportTierView> {
    let mut views: Vec<SeasonalDataExportTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_data_export_tier_view(item));
    }
    views
}

pub fn sum_seasonal_data_export_tier_view(view: &SeasonalDataExportTierView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_margin + (parsed_grace_days as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_data_export_tier("code-615", "title-343");
        let view = to_seasonal_data_export_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_data_export_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
