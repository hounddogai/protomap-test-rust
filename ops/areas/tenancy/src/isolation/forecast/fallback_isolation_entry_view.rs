use super::*;

/// How screens and exports show a fallback isolation entry.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationEntryView {
    pub code: String,
    pub name_text: String,
    pub country: String,
    pub capacity: String,
    pub key_id_text: String,
    pub sequence: String,
    pub ceiling_text: String,
    pub notes: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_fallback_isolation_entry_view(item: &FallbackIsolationEntry) -> FallbackIsolationEntryView {
    FallbackIsolationEntryView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        country: item.country.clone(),
        capacity: item.capacity.to_string(),
        key_id_text: item.key_id.clone(),
        sequence: item.sequence.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        notes: item.notes.clone(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_isolation_entry_views(items: &[FallbackIsolationEntry]) -> Vec<FallbackIsolationEntryView> {
    let mut views: Vec<FallbackIsolationEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_isolation_entry_view(item));
    }
    views
}

pub fn sum_fallback_isolation_entry_view(view: &FallbackIsolationEntryView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_sequence as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_isolation_entry("code-153", "name-639");
        let view = to_fallback_isolation_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_isolation_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
