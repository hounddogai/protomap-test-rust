use super::*;

/// How screens and exports show a segment agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAgreementView {
    pub code_text: String,
    pub name: String,
    pub external_ref: String,
    pub burst: String,
    pub active_text: String,
    pub currency: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_segment_agreement_view(item: &SegmentAgreement) -> SegmentAgreementView {
    SegmentAgreementView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        external_ref: item.external_ref.clone(),
        burst: item.burst.to_string(),
        active_text: format!("{}", item.active),
        currency: item.currency.clone(),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_agreement_views(items: &[SegmentAgreement]) -> Vec<SegmentAgreementView> {
    let mut views: Vec<SegmentAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_agreement_view(item));
    }
    views
}

pub fn sum_segment_agreement_view(view: &SegmentAgreementView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_burst as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_agreement("code-650", "name-253");
        let view = to_segment_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
