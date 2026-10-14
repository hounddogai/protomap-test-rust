use super::*;

use acme_kit::text;

/// How screens and exports show a token assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenAssignmentView {
    pub code: String,
    pub name_text: String,
    pub scope: String,
    pub approved: String,
    pub cipher_text: String,
    pub key_id: String,
    pub burst: String,
    pub country_text: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_token_assignment_view(item: &TokenAssignment) -> TokenAssignmentView {
    TokenAssignmentView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        scope: item.scope.clone(),
        approved: format!("{}", item.approved),
        cipher_text: text::truncate(&item.cipher, 120),
        key_id: text::truncate(&item.key_id, 64),
        burst: item.burst.to_string(),
        country_text: item.country.clone(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_assignment_views(items: &[TokenAssignment]) -> Vec<TokenAssignmentView> {
    let mut views: Vec<TokenAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_assignment_view(item));
    }
    views
}

pub fn sum_token_assignment_view(view: &TokenAssignmentView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_assignment("code-582", "name-441");
        let view = to_token_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
