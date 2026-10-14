use super::*;

/// How screens and exports show a seasonal evidence bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEvidenceBundleView {
    pub code: String,
    pub name: String,
    pub country_text: String,
    pub notes: String,
    pub capacity: String,
    pub issuer: String,
    pub approved: String,
    pub ratio_text: String,
    pub principal: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_seasonal_evidence_bundle_view(item: &SeasonalEvidenceBundle) -> SeasonalEvidenceBundleView {
    SeasonalEvidenceBundleView {
        code: item.code.clone(),
        name: item.name.clone(),
        country_text: item.country.clone(),
        notes: item.notes.clone(),
        capacity: item.capacity.to_string(),
        issuer: item.issuer.clone(),
        approved: format!("{}", item.approved),
        ratio_text: format!("{:.2}", item.ratio),
        principal: item.principal.clone(),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_evidence_bundle_views(items: &[SeasonalEvidenceBundle]) -> Vec<SeasonalEvidenceBundleView> {
    let mut views: Vec<SeasonalEvidenceBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_evidence_bundle_view(item));
    }
    views
}

pub fn sum_seasonal_evidence_bundle_view(view: &SeasonalEvidenceBundleView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_ratio + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_evidence_bundle("code-926", "name-99");
        let view = to_seasonal_evidence_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_evidence_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
