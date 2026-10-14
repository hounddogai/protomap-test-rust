use super::*;

use acme_kit::text;

/// How screens and exports show a freeze target.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTargetView {
    pub code_text: String,
    pub name_text: String,
    pub active: String,
    pub expires_at: String,
    pub label: String,
    pub threshold_text: String,
    pub status_label: String,
}

pub fn to_freeze_target_view(item: &FreezeTarget) -> FreezeTargetView {
    FreezeTargetView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        active: format!("{}", item.active),
        expires_at: item.expires_at.to_string(),
        label: text::truncate(&item.label, 80),
        threshold_text: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_target_views(items: &[FreezeTarget]) -> Vec<FreezeTargetView> {
    let mut views: Vec<FreezeTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_target_view(item));
    }
    views
}

pub fn sum_freeze_target_view(view: &FreezeTargetView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_target("code-794", "name-253");
        let view = to_freeze_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
