use super::*;

/// How screens and exports show a delegation slot.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationSlotView {
    pub code_text: String,
    pub name: String,
    pub notes_text: String,
    pub cipher: String,
    pub ratio: String,
    pub maximum: String,
    pub segment: String,
    pub duration_minutes: String,
    pub host: String,
    pub rate_text: String,
    pub margin_text: String,
    pub status_label: String,
}

pub fn to_delegation_slot_view(item: &DelegationSlot) -> DelegationSlotView {
    DelegationSlotView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        notes_text: item.notes.clone(),
        cipher: item.cipher.clone(),
        ratio: format!("{:.2}", item.ratio),
        maximum: format!("{:.2}", item.maximum),
        segment: item.segment.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        host: item.host.clone(),
        rate_text: format!("{:.2}", item.rate),
        margin_text: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_slot_views(items: &[DelegationSlot]) -> Vec<DelegationSlotView> {
    let mut views: Vec<DelegationSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_slot_view(item));
    }
    views
}

pub fn sum_delegation_slot_view(view: &DelegationSlotView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_ratio + parsed_maximum + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_slot("code-174", "name-158");
        let view = to_delegation_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
