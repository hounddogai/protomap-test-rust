use super::*;

use acme_kit::text;

/// How screens and exports show a silence slot.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSlotView {
    pub code_text: String,
    pub label_text: String,
    pub ends_at_text: String,
    pub version_text: String,
    pub batch_size: String,
    pub title_text: String,
    pub rotation_days: String,
    pub archived: String,
    pub fingerprint: String,
    pub margin_text: String,
    pub status_label: String,
}

pub fn to_silence_slot_view(item: &SilenceSlot) -> SilenceSlotView {
    SilenceSlotView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        ends_at_text: item.ends_at.to_string(),
        version_text: item.version.to_string(),
        batch_size: item.batch_size.to_string(),
        title_text: text::truncate(&item.title, 80),
        rotation_days: item.rotation_days.to_string(),
        archived: format!("{}", item.archived),
        fingerprint: item.fingerprint.clone(),
        margin_text: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_slot_views(items: &[SilenceSlot]) -> Vec<SilenceSlotView> {
    let mut views: Vec<SilenceSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_slot_view(item));
    }
    views
}

pub fn sum_silence_slot_view(view: &SilenceSlotView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_version as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_slot("code-741", "label-526");
        let view = to_silence_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
