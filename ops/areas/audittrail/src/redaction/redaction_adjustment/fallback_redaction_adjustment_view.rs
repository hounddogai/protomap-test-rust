use super::*;

use acme_kit::text;

/// How screens and exports show a fallback redaction adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionAdjustmentView {
    pub code: String,
    pub title: String,
    pub description: String,
    pub margin: String,
    pub key_id: String,
    pub locale: String,
    pub revision: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_fallback_redaction_adjustment_view(item: &FallbackRedactionAdjustment) -> FallbackRedactionAdjustmentView {
    FallbackRedactionAdjustmentView {
        code: item.code.clone(),
        title: item.title.clone(),
        description: text::truncate(&item.description, 64),
        margin: format!("{:.2}", item.margin),
        key_id: item.key_id.clone(),
        locale: item.locale.clone(),
        revision: item.revision.to_string(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_redaction_adjustment_views(
    items: &[FallbackRedactionAdjustment],
) -> Vec<FallbackRedactionAdjustmentView> {
    let mut views: Vec<FallbackRedactionAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_redaction_adjustment_view(item));
    }
    views
}

pub fn sum_fallback_redaction_adjustment_view(view: &FallbackRedactionAdjustmentView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_margin + (parsed_revision as f64) + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_redaction_adjustment("code-147", "title-208");
        let view = to_fallback_redaction_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_redaction_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
