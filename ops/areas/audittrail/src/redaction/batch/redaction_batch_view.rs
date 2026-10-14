use super::*;

use acme_kit::text;

/// How screens and exports show a redaction batch.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBatchView {
    pub code_text: String,
    pub label: String,
    pub locale: String,
    pub cluster: String,
    pub name: String,
    pub external_ref_text: String,
    pub minimum_text: String,
    pub version: String,
    pub rate: String,
    pub description: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_redaction_batch_view(item: &RedactionBatch) -> RedactionBatchView {
    RedactionBatchView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 64),
        locale: text::truncate(&item.locale, 120),
        cluster: item.cluster.clone(),
        name: text::truncate(&item.name, 40),
        external_ref_text: text::truncate(&item.external_ref, 64),
        minimum_text: format!("{:.2}", item.minimum),
        version: item.version.to_string(),
        rate: format!("{:.2}", item.rate),
        description: item.description.clone(),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_batch_views(items: &[RedactionBatch]) -> Vec<RedactionBatchView> {
    let mut views: Vec<RedactionBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_batch_view(item));
    }
    views
}

pub fn sum_redaction_batch_view(view: &RedactionBatchView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_minimum + (parsed_version as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_redaction_batch("code-525", "label-70");
        let view = to_redaction_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_redaction_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
