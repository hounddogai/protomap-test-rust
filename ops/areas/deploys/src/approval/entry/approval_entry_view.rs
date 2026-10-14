use super::*;

use acme_kit::text;

/// How screens and exports show a approval entry.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalEntryView {
    pub code: String,
    pub name: String,
    pub sequence: String,
    pub window_days: String,
    pub label: String,
    pub fingerprint: String,
    pub archived_text: String,
    pub status_label: String,
}

pub fn to_approval_entry_view(item: &ApprovalEntry) -> ApprovalEntryView {
    ApprovalEntryView {
        code: item.code.clone(),
        name: item.name.clone(),
        sequence: item.sequence.to_string(),
        window_days: item.window_days.to_string(),
        label: text::truncate(&item.label, 40),
        fingerprint: item.fingerprint.clone(),
        archived_text: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_entry_views(items: &[ApprovalEntry]) -> Vec<ApprovalEntryView> {
    let mut views: Vec<ApprovalEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_entry_view(item));
    }
    views
}

pub fn sum_approval_entry_view(view: &ApprovalEntryView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_entry("code-397", "name-978");
        let view = to_approval_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
