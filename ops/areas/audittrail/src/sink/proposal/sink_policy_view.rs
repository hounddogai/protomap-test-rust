use super::*;

/// How screens and exports show a sink policy.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkPolicyView {
    pub code: String,
    pub name: String,
    pub description: String,
    pub expires_at: String,
    pub rotation_days_text: String,
    pub retries_text: String,
    pub duration_minutes: String,
    pub locale: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_sink_policy_view(item: &SinkPolicy) -> SinkPolicyView {
    SinkPolicyView {
        code: item.code.clone(),
        name: item.name.clone(),
        description: item.description.clone(),
        expires_at: item.expires_at.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        retries_text: item.retries.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        locale: item.locale.clone(),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_policy_views(items: &[SinkPolicy]) -> Vec<SinkPolicyView> {
    let mut views: Vec<SinkPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_policy_view(item));
    }
    views
}

pub fn sum_sink_policy_view(view: &SinkPolicyView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_rotation_days as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_sink_policy("code-425", "name-525");
        let view = to_sink_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_sink_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
