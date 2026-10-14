use super::*;

use acme_kit::text;

/// How screens and exports show a credential assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAssignmentView {
    pub code_text: String,
    pub name: String,
    pub margin: String,
    pub quantity: String,
    pub ceiling: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_credential_assignment_view(item: &CredentialAssignment) -> CredentialAssignmentView {
    CredentialAssignmentView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 40),
        margin: format!("{:.2}", item.margin),
        quantity: item.quantity.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_assignment_views(items: &[CredentialAssignment]) -> Vec<CredentialAssignmentView> {
    let mut views: Vec<CredentialAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_assignment_view(item));
    }
    views
}

pub fn sum_credential_assignment_view(view: &CredentialAssignmentView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_margin + (parsed_quantity as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_assignment("code-762", "name-174");
        let view = to_credential_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
