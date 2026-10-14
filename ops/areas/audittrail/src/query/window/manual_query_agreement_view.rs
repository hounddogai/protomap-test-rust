use super::*;

use acme_kit::text;

/// How screens and exports show a manual query agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualQueryAgreementView {
    pub code: String,
    pub name: String,
    pub key_id_text: String,
    pub duration_minutes_text: String,
    pub fingerprint: String,
    pub label_text: String,
    pub visible: String,
    pub cost: String,
    pub retries: String,
    pub owner_id: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_manual_query_agreement_view(item: &ManualQueryAgreement) -> ManualQueryAgreementView {
    ManualQueryAgreementView {
        code: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 120),
        key_id_text: item.key_id.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        fingerprint: item.fingerprint.clone(),
        label_text: item.label.clone(),
        visible: format!("{}", item.visible),
        cost: format!("{:.2}", item.cost),
        retries: item.retries.to_string(),
        owner_id: item.owner_id.clone(),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_query_agreement_views(items: &[ManualQueryAgreement]) -> Vec<ManualQueryAgreementView> {
    let mut views: Vec<ManualQueryAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_query_agreement_view(item));
    }
    views
}

pub fn sum_manual_query_agreement_view(view: &ManualQueryAgreementView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_cost + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_query_agreement("code-607", "name-717");
        let view = to_manual_query_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_query_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
