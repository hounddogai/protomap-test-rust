use super::*;

/// How screens and exports show a retention exception.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionExceptionView {
    pub code_text: String,
    pub title: String,
    pub cost: String,
    pub attempts: String,
    pub version_text: String,
    pub archived: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_retention_exception_view(item: &RetentionException) -> RetentionExceptionView {
    RetentionExceptionView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        cost: format!("{:.2}", item.cost),
        attempts: item.attempts.to_string(),
        version_text: item.version.to_string(),
        archived: format!("{}", item.archived),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_exception_views(items: &[RetentionException]) -> Vec<RetentionExceptionView> {
    let mut views: Vec<RetentionExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_exception_view(item));
    }
    views
}

pub fn sum_retention_exception_view(view: &RetentionExceptionView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_cost + (parsed_attempts as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_exception("code-254", "title-506");
        let view = to_retention_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
