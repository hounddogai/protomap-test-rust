use super::*;

use acme_kit::text;

/// How screens and exports show a runbook variance.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookVarianceView {
    pub code_text: String,
    pub title: String,
    pub external_ref_text: String,
    pub audience: String,
    pub rate_text: String,
    pub weight: String,
    pub notes: String,
    pub margin: String,
    pub minimum: String,
    pub maximum: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_runbook_variance_view(item: &RunbookVariance) -> RunbookVarianceView {
    RunbookVarianceView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        external_ref_text: item.external_ref.clone(),
        audience: item.audience.clone(),
        rate_text: format!("{:.2}", item.rate),
        weight: format!("{:.2}", item.weight),
        notes: text::truncate(&item.notes, 40),
        margin: format!("{:.2}", item.margin),
        minimum: format!("{:.2}", item.minimum),
        maximum: format!("{:.2}", item.maximum),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_variance_views(items: &[RunbookVariance]) -> Vec<RunbookVarianceView> {
    let mut views: Vec<RunbookVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_variance_view(item));
    }
    views
}

pub fn sum_runbook_variance_view(view: &RunbookVarianceView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_rate + parsed_weight + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_variance("code-148", "title-972");
        let view = to_runbook_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
