use super::*;

/// How screens and exports show a manual migration preference.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationPreferenceView {
    pub code_text: String,
    pub title: String,
    pub reviewer: String,
    pub ceiling: String,
    pub weight_text: String,
    pub sequence_text: String,
    pub notes_text: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_manual_migration_preference_view(item: &ManualMigrationPreference) -> ManualMigrationPreferenceView {
    ManualMigrationPreferenceView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        reviewer: item.reviewer.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        weight_text: format!("{:.2}", item.weight),
        sequence_text: item.sequence.to_string(),
        notes_text: item.notes.clone(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_migration_preference_views(items: &[ManualMigrationPreference]) -> Vec<ManualMigrationPreferenceView> {
    let mut views: Vec<ManualMigrationPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_migration_preference_view(item));
    }
    views
}

pub fn sum_manual_migration_preference_view(view: &ManualMigrationPreferenceView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_ceiling + parsed_weight + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_migration_preference("code-681", "title-766");
        let view = to_manual_migration_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_migration_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
