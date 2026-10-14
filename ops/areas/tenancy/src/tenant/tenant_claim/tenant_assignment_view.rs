use super::*;

use acme_kit::text;

/// How screens and exports show a tenant assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantAssignmentView {
    pub code: String,
    pub label: String,
    pub cluster_text: String,
    pub title_text: String,
    pub sequence_text: String,
    pub fingerprint: String,
    pub scope_text: String,
    pub status_label: String,
}

pub fn to_tenant_assignment_view(item: &TenantAssignment) -> TenantAssignmentView {
    TenantAssignmentView {
        code: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 64),
        cluster_text: item.cluster.clone(),
        title_text: item.title.clone(),
        sequence_text: item.sequence.to_string(),
        fingerprint: item.fingerprint.clone(),
        scope_text: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_assignment_views(items: &[TenantAssignment]) -> Vec<TenantAssignmentView> {
    let mut views: Vec<TenantAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_assignment_view(item));
    }
    views
}

pub fn sum_tenant_assignment_view(view: &TenantAssignmentView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_sequence as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_assignment("code-119", "label-748");
        let view = to_tenant_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
