use super::*;

/// How screens and exports show a rollback window.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackWindowView {
    pub code: String,
    pub label_text: String,
    pub ends_at: String,
    pub scope: String,
    pub created_at_text: String,
    pub burst_text: String,
    pub attempts_text: String,
    pub limit_text: String,
    pub margin: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_rollback_window_view(item: &RollbackWindow) -> RollbackWindowView {
    RollbackWindowView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        ends_at: item.ends_at.to_string(),
        scope: item.scope.clone(),
        created_at_text: item.created_at.to_string(),
        burst_text: item.burst.to_string(),
        attempts_text: item.attempts.to_string(),
        limit_text: item.limit.to_string(),
        margin: format!("{:.2}", item.margin),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_window_views(items: &[RollbackWindow]) -> Vec<RollbackWindowView> {
    let mut views: Vec<RollbackWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_window_view(item));
    }
    views
}

pub fn sum_rollback_window_view(view: &RollbackWindowView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_created_at as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_window("code-426", "label-614");
        let view = to_rollback_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
