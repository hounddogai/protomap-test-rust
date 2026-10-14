use super::*;

/// How screens and exports show a silence segment.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSegmentView {
    pub code: String,
    pub label_text: String,
    pub version: String,
    pub host: String,
    pub discount: String,
    pub amount: String,
    pub audience_text: String,
    pub external_ref_text: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_silence_segment_view(item: &SilenceSegment) -> SilenceSegmentView {
    SilenceSegmentView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        version: item.version.to_string(),
        host: item.host.clone(),
        discount: format!("{:.2}", item.discount),
        amount: format!("{:.2}", item.amount),
        audience_text: item.audience.clone(),
        external_ref_text: item.external_ref.clone(),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_segment_views(items: &[SilenceSegment]) -> Vec<SilenceSegmentView> {
    let mut views: Vec<SilenceSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_segment_view(item));
    }
    views
}

pub fn sum_silence_segment_view(view: &SilenceSegmentView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_version as f64 + parsed_discount + parsed_amount)
}
