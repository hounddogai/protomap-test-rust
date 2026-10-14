use super::*;

use acme_kit::text;

/// How screens and exports show a freeze record.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRecordView {
    pub code: String,
    pub label: String,
    pub discount: String,
    pub limit_text: String,
    pub description_text: String,
    pub updated_at: String,
    pub reviewer: String,
    pub issuer: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_freeze_record_view(item: &FreezeRecord) -> FreezeRecordView {
    FreezeRecordView {
        code: item.code.clone(),
        label: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        limit_text: item.limit.to_string(),
        description_text: text::truncate(&item.description, 120),
        updated_at: item.updated_at.to_string(),
        reviewer: item.reviewer.clone(),
        issuer: item.issuer.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_record_views(items: &[FreezeRecord]) -> Vec<FreezeRecordView> {
    let mut views: Vec<FreezeRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_record_view(item));
    }
    views
}

pub fn sum_freeze_record_view(view: &FreezeRecordView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_discount + (parsed_limit as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_record("code-849", "label-111");
        let view = to_freeze_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
