use super::*;

/// How screens and exports show a limit group.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitGroupView {
    pub code: String,
    pub title: String,
    pub port: String,
    pub name: String,
    pub fingerprint: String,
    pub quota_units_text: String,
    pub budget: String,
    pub cipher_text: String,
    pub contact_email_text: String,
    pub notes: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_limit_group_view(item: &LimitGroup) -> LimitGroupView {
    LimitGroupView {
        code: item.code.clone(),
        title: item.title.clone(),
        port: item.port.to_string(),
        name: item.name.clone(),
        fingerprint: item.fingerprint.clone(),
        quota_units_text: item.quota_units.to_string(),
        budget: format!("{:.2}", item.budget),
        cipher_text: item.cipher.clone(),
        contact_email_text: item.contact_email.clone(),
        notes: item.notes.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_group_views(items: &[LimitGroup]) -> Vec<LimitGroupView> {
    let mut views: Vec<LimitGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_group_view(item));
    }
    views
}

pub fn sum_limit_group_view(view: &LimitGroupView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_port as f64 + (parsed_quota_units as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_group("code-101", "title-316");
        let view = to_limit_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
