use super::*;

use acme_kit::text;

/// How screens and exports show a capped forecast claim.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedForecastClaimView {
    pub code_text: String,
    pub label: String,
    pub source: String,
    pub updated_at_text: String,
    pub audience: String,
    pub notes_text: String,
    pub quantity: String,
    pub tenant_id: String,
    pub score_text: String,
    pub scope: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_capped_forecast_claim_view(item: &CappedForecastClaim) -> CappedForecastClaimView {
    CappedForecastClaimView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        source: text::truncate(&item.source, 120),
        updated_at_text: item.updated_at.to_string(),
        audience: item.audience.clone(),
        notes_text: text::truncate(&item.notes, 64),
        quantity: item.quantity.to_string(),
        tenant_id: text::truncate(&item.tenant_id, 120),
        score_text: format!("{:.2}", item.score),
        scope: item.scope.clone(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_forecast_claim_views(items: &[CappedForecastClaim]) -> Vec<CappedForecastClaimView> {
    let mut views: Vec<CappedForecastClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_forecast_claim_view(item));
    }
    views
}

pub fn sum_capped_forecast_claim_view(view: &CappedForecastClaimView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_quantity as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_forecast_claim("code-994", "label-725");
        let view = to_capped_forecast_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_forecast_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
