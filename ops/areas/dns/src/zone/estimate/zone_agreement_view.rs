use super::*;

/// How screens and exports show a zone agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAgreementView {
    pub code_text: String,
    pub title: String,
    pub locale: String,
    pub updated_at: String,
    pub external_ref_text: String,
    pub retries_text: String,
    pub sequence: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_zone_agreement_view(item: &ZoneAgreement) -> ZoneAgreementView {
    ZoneAgreementView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        locale: item.locale.clone(),
        updated_at: item.updated_at.to_string(),
        external_ref_text: item.external_ref.clone(),
        retries_text: item.retries.to_string(),
        sequence: item.sequence.to_string(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_agreement_views(items: &[ZoneAgreement]) -> Vec<ZoneAgreementView> {
    let mut views: Vec<ZoneAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_agreement_view(item));
    }
    views
}

pub fn sum_zone_agreement_view(view: &ZoneAgreementView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_retries as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_agreement("code-989", "title-26");
        let view = to_zone_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
