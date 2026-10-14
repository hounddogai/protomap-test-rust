use super::*;

use acme_kit::text;

/// How screens and exports show a manual redaction target.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionTargetView {
    pub code: String,
    pub label: String,
    pub updated_at: String,
    pub limit: String,
    pub audience: String,
    pub locale_text: String,
    pub minimum: String,
    pub created_at: String,
    pub unit_text: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_manual_redaction_target_view(item: &ManualRedactionTarget) -> ManualRedactionTargetView {
    ManualRedactionTargetView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        updated_at: item.updated_at.to_string(),
        limit: item.limit.to_string(),
        audience: item.audience.clone(),
        locale_text: item.locale.clone(),
        minimum: format!("{:.2}", item.minimum),
        created_at: item.created_at.to_string(),
        unit_text: item.unit.clone(),
        phone: text::truncate(&item.phone, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_redaction_target_views(items: &[ManualRedactionTarget]) -> Vec<ManualRedactionTargetView> {
    let mut views: Vec<ManualRedactionTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_redaction_target_view(item));
    }
    views
}

pub fn sum_manual_redaction_target_view(view: &ManualRedactionTargetView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_limit as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_redaction_target("code-393", "label-102");
        let view = to_manual_redaction_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_redaction_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
