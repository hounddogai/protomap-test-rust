use super::*;

use acme_kit::text;

/// How screens and exports show a pending binding window.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingWindowView {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub rotation_days: String,
    pub issuer: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_pending_binding_window_view(item: &PendingBindingWindow) -> PendingBindingWindowView {
    PendingBindingWindowView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        notes: item.notes.clone(),
        rotation_days: item.rotation_days.to_string(),
        issuer: text::truncate(&item.issuer, 80),
        host: text::truncate(&item.host, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_binding_window_views(items: &[PendingBindingWindow]) -> Vec<PendingBindingWindowView> {
    let mut views: Vec<PendingBindingWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_binding_window_view(item));
    }
    views
}

pub fn sum_pending_binding_window_view(view: &PendingBindingWindowView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_rotation_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_binding_window("code-836", "name-770");
        let view = to_pending_binding_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_binding_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
