use super::*;

use acme_kit::text;

/// How screens and exports show a rule batch.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleBatchView {
    pub code: String,
    pub name_text: String,
    pub locale: String,
    pub duration_minutes_text: String,
    pub checksum: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_rule_batch_view(item: &RuleBatch) -> RuleBatchView {
    RuleBatchView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        locale: item.locale.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        checksum: text::truncate(&item.checksum, 40),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_batch_views(items: &[RuleBatch]) -> Vec<RuleBatchView> {
    let mut views: Vec<RuleBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_batch_view(item));
    }
    views
}

pub fn sum_rule_batch_view(view: &RuleBatchView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_batch("code-472", "name-858");
        let view = to_rule_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
