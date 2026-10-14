use super::*;

/// How screens and exports show a scheduled credential window.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCredentialWindowView {
    pub code: String,
    pub label: String,
    pub approved: String,
    pub rotation_days: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_scheduled_credential_window_view(item: &ScheduledCredentialWindow) -> ScheduledCredentialWindowView {
    ScheduledCredentialWindowView {
        code: item.code.clone(),
        label: item.label.clone(),
        approved: format!("{}", item.approved),
        rotation_days: item.rotation_days.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_credential_window_views(items: &[ScheduledCredentialWindow]) -> Vec<ScheduledCredentialWindowView> {
    let mut views: Vec<ScheduledCredentialWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_credential_window_view(item));
    }
    views
}

pub fn sum_scheduled_credential_window_view(view: &ScheduledCredentialWindowView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_credential_window("code-188", "label-290");
        let view = to_scheduled_credential_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_credential_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
