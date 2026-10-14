use super::*;

/// How screens and exports show a migration milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationMilestoneView {
    pub code: String,
    pub name: String,
    pub cipher_text: String,
    pub ratio: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_migration_milestone_view(item: &MigrationMilestone) -> MigrationMilestoneView {
    MigrationMilestoneView {
        code: item.code.clone(),
        name: item.name.clone(),
        cipher_text: item.cipher.clone(),
        ratio: format!("{:.2}", item.ratio),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_milestone_views(items: &[MigrationMilestone]) -> Vec<MigrationMilestoneView> {
    let mut views: Vec<MigrationMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_milestone_view(item));
    }
    views
}

pub fn sum_migration_milestone_view(view: &MigrationMilestoneView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_ratio + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_milestone("code-823", "name-726");
        let view = to_migration_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
