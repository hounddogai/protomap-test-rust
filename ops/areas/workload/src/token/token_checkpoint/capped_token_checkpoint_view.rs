use super::*;

/// How screens and exports show a capped token checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenCheckpointView {
    pub code: String,
    pub title_text: String,
    pub notes_text: String,
    pub expires_at_text: String,
    pub weight: String,
    pub contact_email: String,
    pub threshold: String,
    pub rate: String,
    pub capacity: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_capped_token_checkpoint_view(item: &CappedTokenCheckpoint) -> CappedTokenCheckpointView {
    CappedTokenCheckpointView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        notes_text: item.notes.clone(),
        expires_at_text: item.expires_at.to_string(),
        weight: format!("{:.2}", item.weight),
        contact_email: item.contact_email.clone(),
        threshold: format!("{:.2}", item.threshold),
        rate: format!("{:.2}", item.rate),
        capacity: item.capacity.to_string(),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_token_checkpoint_views(items: &[CappedTokenCheckpoint]) -> Vec<CappedTokenCheckpointView> {
    let mut views: Vec<CappedTokenCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_token_checkpoint_view(item));
    }
    views
}

pub fn sum_capped_token_checkpoint_view(view: &CappedTokenCheckpointView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_weight + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_token_checkpoint("code-132", "title-77");
        let view = to_capped_token_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_token_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
