use super::*;

use acme_kit::text;

/// How screens and exports show a regional verifier batch.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierBatchView {
    pub code: String,
    pub label: String,
    pub version_text: String,
    pub ceiling_text: String,
    pub region_text: String,
    pub margin: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_regional_verifier_batch_view(item: &RegionalVerifierBatch) -> RegionalVerifierBatchView {
    RegionalVerifierBatchView {
        code: item.code.clone(),
        label: item.label.clone(),
        version_text: item.version.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        region_text: item.region.clone(),
        margin: format!("{:.2}", item.margin),
        principal: text::truncate(&item.principal, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_verifier_batch_views(items: &[RegionalVerifierBatch]) -> Vec<RegionalVerifierBatchView> {
    let mut views: Vec<RegionalVerifierBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_verifier_batch_view(item));
    }
    views
}

pub fn sum_regional_verifier_batch_view(view: &RegionalVerifierBatchView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_version as f64 + parsed_ceiling + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_verifier_batch("code-423", "label-949");
        let view = to_regional_verifier_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_verifier_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
