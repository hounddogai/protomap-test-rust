use super::*;

/// How screens and exports show a credential batch.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBatchView {
    pub code_text: String,
    pub name: String,
    pub title_text: String,
    pub quantity: String,
    pub rotation_days_text: String,
    pub status_label: String,
}

pub fn to_credential_batch_view(item: &CredentialBatch) -> CredentialBatchView {
    CredentialBatchView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        title_text: item.title.clone(),
        quantity: item.quantity.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_batch_views(items: &[CredentialBatch]) -> Vec<CredentialBatchView> {
    let mut views: Vec<CredentialBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_batch_view(item));
    }
    views
}

pub fn sum_credential_batch_view(view: &CredentialBatchView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_batch("code-35", "name-39");
        let view = to_credential_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
