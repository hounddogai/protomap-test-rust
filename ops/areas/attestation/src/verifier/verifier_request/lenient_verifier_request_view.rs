use super::*;

use acme_kit::text;

/// How screens and exports show a lenient verifier request.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientVerifierRequestView {
    pub code_text: String,
    pub label: String,
    pub discount_text: String,
    pub threshold_text: String,
    pub external_ref: String,
    pub rate: String,
    pub cipher_text: String,
    pub status_label: String,
}

pub fn to_lenient_verifier_request_view(item: &LenientVerifierRequest) -> LenientVerifierRequestView {
    LenientVerifierRequestView {
        code_text: text::truncate(&item.code, 120),
        label: item.label.clone(),
        discount_text: format!("{:.2}", item.discount),
        threshold_text: format!("{:.2}", item.threshold),
        external_ref: item.external_ref.clone(),
        rate: format!("{:.2}", item.rate),
        cipher_text: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_verifier_request_views(items: &[LenientVerifierRequest]) -> Vec<LenientVerifierRequestView> {
    let mut views: Vec<LenientVerifierRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_verifier_request_view(item));
    }
    views
}

pub fn sum_lenient_verifier_request_view(view: &LenientVerifierRequestView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_discount + parsed_threshold + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_verifier_request("code-358", "label-77");
        let view = to_lenient_verifier_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_verifier_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
