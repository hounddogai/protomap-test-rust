use super::*;

use acme_kit::text;

/// How screens and exports show a rule exception.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleExceptionView {
    pub code: String,
    pub title_text: String,
    pub summary: String,
    pub contact_email: String,
    pub starts_at_text: String,
    pub ratio: String,
    pub score: String,
    pub margin: String,
    pub rotation_days_text: String,
    pub status_label: String,
}

pub fn to_rule_exception_view(item: &RuleException) -> RuleExceptionView {
    RuleExceptionView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        summary: text::truncate(&item.summary, 80),
        contact_email: item.contact_email.clone(),
        starts_at_text: item.starts_at.to_string(),
        ratio: format!("{:.2}", item.ratio),
        score: format!("{:.2}", item.score),
        margin: format!("{:.2}", item.margin),
        rotation_days_text: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_exception_views(items: &[RuleException]) -> Vec<RuleExceptionView> {
    let mut views: Vec<RuleExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_exception_view(item));
    }
    views
}

pub fn sum_rule_exception_view(view: &RuleExceptionView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_ratio + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_exception("code-242", "title-13");
        let view = to_rule_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
