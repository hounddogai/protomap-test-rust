use super::*;

use acme_kit::text;

/// How screens and exports show a strict policy snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicySnapshotView {
    pub code_text: String,
    pub name_text: String,
    pub port: String,
    pub margin: String,
    pub priority: String,
    pub cipher: String,
    pub expires_at: String,
    pub approved_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_strict_policy_snapshot_view(item: &StrictPolicySnapshot) -> StrictPolicySnapshotView {
    StrictPolicySnapshotView {
        code_text: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        port: item.port.to_string(),
        margin: format!("{:.2}", item.margin),
        priority: item.priority.to_string(),
        cipher: item.cipher.clone(),
        expires_at: item.expires_at.to_string(),
        approved_text: format!("{}", item.approved),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_policy_snapshot_views(items: &[StrictPolicySnapshot]) -> Vec<StrictPolicySnapshotView> {
    let mut views: Vec<StrictPolicySnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_policy_snapshot_view(item));
    }
    views
}

pub fn sum_strict_policy_snapshot_view(view: &StrictPolicySnapshotView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_port as f64 + parsed_margin + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_policy_snapshot("code-165", "name-95");
        let view = to_strict_policy_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_policy_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
