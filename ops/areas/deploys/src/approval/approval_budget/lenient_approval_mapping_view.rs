use super::*;

use acme_kit::text;

/// How screens and exports show a lenient approval mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientApprovalMappingView {
    pub code: String,
    pub label_text: String,
    pub cipher_text: String,
    pub sequence: String,
    pub ratio: String,
    pub margin: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_lenient_approval_mapping_view(item: &LenientApprovalMapping) -> LenientApprovalMappingView {
    LenientApprovalMappingView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        cipher_text: text::truncate(&item.cipher, 80),
        sequence: item.sequence.to_string(),
        ratio: format!("{:.2}", item.ratio),
        margin: format!("{:.2}", item.margin),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_approval_mapping_views(items: &[LenientApprovalMapping]) -> Vec<LenientApprovalMappingView> {
    let mut views: Vec<LenientApprovalMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_approval_mapping_view(item));
    }
    views
}

pub fn sum_lenient_approval_mapping_view(view: &LenientApprovalMappingView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_ratio + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_approval_mapping("code-50", "label-544");
        let view = to_lenient_approval_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_approval_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
