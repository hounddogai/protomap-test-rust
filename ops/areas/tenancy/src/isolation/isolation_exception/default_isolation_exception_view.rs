use super::*;

use acme_kit::text;

/// How screens and exports show a default isolation exception.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIsolationExceptionView {
    pub code: String,
    pub title: String,
    pub priority: String,
    pub attempts_text: String,
    pub segment: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_default_isolation_exception_view(item: &DefaultIsolationException) -> DefaultIsolationExceptionView {
    DefaultIsolationExceptionView {
        code: text::truncate(&item.code, 40),
        title: text::truncate(&item.title, 64),
        priority: item.priority.to_string(),
        attempts_text: item.attempts.to_string(),
        segment: text::truncate(&item.segment, 120),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_isolation_exception_views(items: &[DefaultIsolationException]) -> Vec<DefaultIsolationExceptionView> {
    let mut views: Vec<DefaultIsolationExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_isolation_exception_view(item));
    }
    views
}

pub fn sum_default_isolation_exception_view(view: &DefaultIsolationExceptionView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_attempts as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_isolation_exception("code-215", "title-476");
        let view = to_default_isolation_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_isolation_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
