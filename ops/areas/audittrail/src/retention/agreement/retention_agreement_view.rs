use super::*;

use acme_kit::text;

/// How screens and exports show a retention agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAgreementView {
    pub code_text: String,
    pub label_text: String,
    pub minimum: String,
    pub margin: String,
    pub principal: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_retention_agreement_view(item: &RetentionAgreement) -> RetentionAgreementView {
    RetentionAgreementView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        minimum: format!("{:.2}", item.minimum),
        margin: format!("{:.2}", item.margin),
        principal: item.principal.clone(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_agreement_views(items: &[RetentionAgreement]) -> Vec<RetentionAgreementView> {
    let mut views: Vec<RetentionAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_agreement_view(item));
    }
    views
}

pub fn sum_retention_agreement_view(view: &RetentionAgreementView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_minimum + parsed_margin + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_agreement("code-754", "label-508");
        let view = to_retention_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
