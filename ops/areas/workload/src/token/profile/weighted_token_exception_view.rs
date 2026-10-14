use super::*;

use acme_kit::text;

/// How screens and exports show a weighted token exception.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenExceptionView {
    pub code: String,
    pub label: String,
    pub batch_size: String,
    pub minimum_text: String,
    pub priority_text: String,
    pub revision_text: String,
    pub scope: String,
    pub description: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_weighted_token_exception_view(item: &WeightedTokenException) -> WeightedTokenExceptionView {
    WeightedTokenExceptionView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        batch_size: item.batch_size.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        priority_text: item.priority.to_string(),
        revision_text: item.revision.to_string(),
        scope: item.scope.clone(),
        description: text::truncate(&item.description, 120),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_token_exception_views(items: &[WeightedTokenException]) -> Vec<WeightedTokenExceptionView> {
    let mut views: Vec<WeightedTokenExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_token_exception_view(item));
    }
    views
}

pub fn sum_weighted_token_exception_view(view: &WeightedTokenExceptionView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_minimum + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_token_exception("code-280", "label-542");
        let view = to_weighted_token_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_token_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
