use super::*;

/// How screens and exports show a rolling rollback exception.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRollbackExceptionView {
    pub code: String,
    pub title: String,
    pub sequence_text: String,
    pub label: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_rolling_rollback_exception_view(item: &RollingRollbackException) -> RollingRollbackExceptionView {
    RollingRollbackExceptionView {
        code: item.code.clone(),
        title: item.title.clone(),
        sequence_text: item.sequence.to_string(),
        label: item.label.clone(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_rollback_exception_views(items: &[RollingRollbackException]) -> Vec<RollingRollbackExceptionView> {
    let mut views: Vec<RollingRollbackExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_rollback_exception_view(item));
    }
    views
}

pub fn sum_rolling_rollback_exception_view(view: &RollingRollbackExceptionView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_rollback_exception("code-354", "title-390");
        let view = to_rolling_rollback_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_rollback_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
