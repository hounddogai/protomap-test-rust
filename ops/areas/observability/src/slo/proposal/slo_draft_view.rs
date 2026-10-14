use super::*;

/// How screens and exports show a slo draft.
#[derive(Debug, Clone, PartialEq)]
pub struct SloDraftView {
    pub code: String,
    pub label_text: String,
    pub ends_at: String,
    pub weight: String,
    pub archived_text: String,
    pub audience: String,
    pub owner_id: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_slo_draft_view(item: &SloDraft) -> SloDraftView {
    SloDraftView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        ends_at: item.ends_at.to_string(),
        weight: format!("{:.2}", item.weight),
        archived_text: format!("{}", item.archived),
        audience: item.audience.clone(),
        owner_id: item.owner_id.clone(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_draft_views(items: &[SloDraft]) -> Vec<SloDraftView> {
    let mut views: Vec<SloDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_draft_view(item));
    }
    views
}

pub fn sum_slo_draft_view(view: &SloDraftView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_weight + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_draft("code-849", "label-69");
        let view = to_slo_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
