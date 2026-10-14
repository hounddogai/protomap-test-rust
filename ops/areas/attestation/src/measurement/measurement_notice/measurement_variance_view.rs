use super::*;

use acme_kit::text;

/// How screens and exports show a measurement variance.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementVarianceView {
    pub code_text: String,
    pub label: String,
    pub audience: String,
    pub issuer: String,
    pub currency: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_measurement_variance_view(item: &MeasurementVariance) -> MeasurementVarianceView {
    MeasurementVarianceView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        audience: text::truncate(&item.audience, 120),
        issuer: text::truncate(&item.issuer, 80),
        currency: text::truncate(&item.currency, 80),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_measurement_variance_views(items: &[MeasurementVariance]) -> Vec<MeasurementVarianceView> {
    let mut views: Vec<MeasurementVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_measurement_variance_view(item));
    }
    views
}

pub fn sum_measurement_variance_view(view: &MeasurementVarianceView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_measurement_variance("code-774", "label-850");
        let view = to_measurement_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_measurement_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
