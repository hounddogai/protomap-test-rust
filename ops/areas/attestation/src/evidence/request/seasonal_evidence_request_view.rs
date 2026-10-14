use super::*;

/// How screens and exports show a seasonal evidence request.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEvidenceRequestView {
    pub code: String,
    pub name: String,
    pub pinned: String,
    pub cipher: String,
    pub updated_at: String,
    pub starts_at: String,
    pub segment: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_seasonal_evidence_request_view(item: &SeasonalEvidenceRequest) -> SeasonalEvidenceRequestView {
    SeasonalEvidenceRequestView {
        code: item.code.clone(),
        name: item.name.clone(),
        pinned: format!("{}", item.pinned),
        cipher: item.cipher.clone(),
        updated_at: item.updated_at.to_string(),
        starts_at: item.starts_at.to_string(),
        segment: item.segment.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_evidence_request_views(items: &[SeasonalEvidenceRequest]) -> Vec<SeasonalEvidenceRequestView> {
    let mut views: Vec<SeasonalEvidenceRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_evidence_request_view(item));
    }
    views
}

pub fn sum_seasonal_evidence_request_view(view: &SeasonalEvidenceRequestView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_starts_at as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_evidence_request("code-499", "name-835");
        let view = to_seasonal_evidence_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_evidence_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
