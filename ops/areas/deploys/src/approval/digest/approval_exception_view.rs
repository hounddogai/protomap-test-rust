use super::*;

/// How screens and exports show a approval exception.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalExceptionView {
    pub code_text: String,
    pub title: String,
    pub country: String,
    pub host_text: String,
    pub attempts: String,
    pub timezone: String,
    pub discount_text: String,
    pub status_label: String,
}

pub fn to_approval_exception_view(item: &ApprovalException) -> ApprovalExceptionView {
    ApprovalExceptionView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        country: item.country.clone(),
        host_text: item.host.clone(),
        attempts: item.attempts.to_string(),
        timezone: item.timezone.clone(),
        discount_text: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_exception_views(items: &[ApprovalException]) -> Vec<ApprovalExceptionView> {
    let mut views: Vec<ApprovalExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_exception_view(item));
    }
    views
}

pub fn sum_approval_exception_view(view: &ApprovalExceptionView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_exception("code-987", "title-253");
        let view = to_approval_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
