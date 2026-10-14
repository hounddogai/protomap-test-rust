use super::*;

use acme_kit::text;

/// How screens and exports show a token slot.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSlotView {
    pub code_text: String,
    pub label: String,
    pub duration_minutes: String,
    pub window_days: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_token_slot_view(item: &TokenSlot) -> TokenSlotView {
    TokenSlotView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 120),
        duration_minutes: item.duration_minutes.to_string(),
        window_days: item.window_days.to_string(),
        notes: text::truncate(&item.notes, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_slot_views(items: &[TokenSlot]) -> Vec<TokenSlotView> {
    let mut views: Vec<TokenSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_slot_view(item));
    }
    views
}

pub fn sum_token_slot_view(view: &TokenSlotView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_slot("code-343", "label-968");
        let view = to_token_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
