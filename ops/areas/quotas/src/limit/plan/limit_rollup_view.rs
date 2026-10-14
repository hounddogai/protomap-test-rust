use super::*;

use acme_kit::text;

/// How screens and exports show a limit rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitRollupView {
    pub code: String,
    pub label_text: String,
    pub archived: String,
    pub created_at: String,
    pub ends_at: String,
    pub threshold: String,
    pub quantity: String,
    pub visible: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_limit_rollup_view(item: &LimitRollup) -> LimitRollupView {
    LimitRollupView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        archived: format!("{}", item.archived),
        created_at: item.created_at.to_string(),
        ends_at: item.ends_at.to_string(),
        threshold: format!("{:.2}", item.threshold),
        quantity: item.quantity.to_string(),
        visible: format!("{}", item.visible),
        cipher: text::truncate(&item.cipher, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_rollup_views(items: &[LimitRollup]) -> Vec<LimitRollupView> {
    let mut views: Vec<LimitRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_rollup_view(item));
    }
    views
}

pub fn sum_limit_rollup_view(view: &LimitRollupView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_ends_at as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_rollup("code-598", "label-671");
        let view = to_limit_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
