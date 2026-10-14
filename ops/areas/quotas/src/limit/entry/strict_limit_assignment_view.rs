use super::*;

/// How screens and exports show a strict limit assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictLimitAssignmentView {
    pub code: String,
    pub title: String,
    pub ceiling: String,
    pub tenant_id: String,
    pub archived: String,
    pub sequence: String,
    pub cipher_text: String,
    pub status_label: String,
}

pub fn to_strict_limit_assignment_view(item: &StrictLimitAssignment) -> StrictLimitAssignmentView {
    StrictLimitAssignmentView {
        code: item.code.clone(),
        title: item.title.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        tenant_id: item.tenant_id.clone(),
        archived: format!("{}", item.archived),
        sequence: item.sequence.to_string(),
        cipher_text: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_limit_assignment_views(items: &[StrictLimitAssignment]) -> Vec<StrictLimitAssignmentView> {
    let mut views: Vec<StrictLimitAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_limit_assignment_view(item));
    }
    views
}

pub fn sum_strict_limit_assignment_view(view: &StrictLimitAssignmentView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_ceiling + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_limit_assignment("code-490", "title-554");
        let view = to_strict_limit_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_limit_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
