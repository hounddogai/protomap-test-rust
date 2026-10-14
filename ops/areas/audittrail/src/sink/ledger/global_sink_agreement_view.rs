use super::*;

use acme_kit::text;

/// How screens and exports show a global sink agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkAgreementView {
    pub code: String,
    pub name: String,
    pub fingerprint_text: String,
    pub maximum: String,
    pub score: String,
    pub batch_size: String,
    pub audience: String,
    pub grace_days_text: String,
    pub minimum_text: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_global_sink_agreement_view(item: &GlobalSinkAgreement) -> GlobalSinkAgreementView {
    GlobalSinkAgreementView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        fingerprint_text: item.fingerprint.clone(),
        maximum: format!("{:.2}", item.maximum),
        score: format!("{:.2}", item.score),
        batch_size: item.batch_size.to_string(),
        audience: item.audience.clone(),
        grace_days_text: item.grace_days.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_sink_agreement_views(items: &[GlobalSinkAgreement]) -> Vec<GlobalSinkAgreementView> {
    let mut views: Vec<GlobalSinkAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_sink_agreement_view(item));
    }
    views
}

pub fn sum_global_sink_agreement_view(view: &GlobalSinkAgreementView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_maximum + parsed_score + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_sink_agreement("code-731", "name-912");
        let view = to_global_sink_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_sink_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
