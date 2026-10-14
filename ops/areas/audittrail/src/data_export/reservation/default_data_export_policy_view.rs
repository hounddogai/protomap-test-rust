use super::*;

use acme_kit::text;

/// How screens and exports show a default data export policy.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportPolicyView {
    pub code: String,
    pub title_text: String,
    pub contact_email: String,
    pub scope: String,
    pub label: String,
    pub issuer: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_default_data_export_policy_view(item: &DefaultDataExportPolicy) -> DefaultDataExportPolicyView {
    DefaultDataExportPolicyView {
        code: text::truncate(&item.code, 40),
        title_text: text::truncate(&item.title, 120),
        contact_email: item.contact_email.clone(),
        scope: item.scope.clone(),
        label: item.label.clone(),
        issuer: item.issuer.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_data_export_policy_views(items: &[DefaultDataExportPolicy]) -> Vec<DefaultDataExportPolicyView> {
    let mut views: Vec<DefaultDataExportPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_data_export_policy_view(item));
    }
    views
}

pub fn sum_default_data_export_policy_view(view: &DefaultDataExportPolicyView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_data_export_policy("code-982", "title-460");
        let view = to_default_data_export_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_data_export_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
