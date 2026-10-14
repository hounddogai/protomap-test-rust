use super::*;

use acme_kit::text;

/// How screens and exports show a token checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenCheckpointView {
    pub code_text: String,
    pub title_text: String,
    pub window_days_text: String,
    pub floor_value: String,
    pub retries: String,
    pub duration_minutes: String,
    pub key_id: String,
    pub cipher: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_token_checkpoint_view(item: &TokenCheckpoint) -> TokenCheckpointView {
    TokenCheckpointView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        window_days_text: item.window_days.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        retries: item.retries.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        key_id: item.key_id.clone(),
        cipher: item.cipher.clone(),
        fingerprint: text::truncate(&item.fingerprint, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_checkpoint_views(items: &[TokenCheckpoint]) -> Vec<TokenCheckpointView> {
    let mut views: Vec<TokenCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_checkpoint_view(item));
    }
    views
}

pub fn sum_token_checkpoint_view(view: &TokenCheckpointView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_floor_value + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_checkpoint("code-657", "title-105");
        let view = to_token_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
