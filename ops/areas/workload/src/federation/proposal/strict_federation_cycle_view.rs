use super::*;

/// How screens and exports show a strict federation cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationCycleView {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub created_at: String,
    pub notes: String,
    pub checksum: String,
    pub summary: String,
    pub grace_days_text: String,
    pub name: String,
    pub host: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_strict_federation_cycle_view(item: &StrictFederationCycle) -> StrictFederationCycleView {
    StrictFederationCycleView {
        code: item.code.clone(),
        title: item.title.clone(),
        currency: item.currency.clone(),
        created_at: item.created_at.to_string(),
        notes: item.notes.clone(),
        checksum: item.checksum.clone(),
        summary: item.summary.clone(),
        grace_days_text: item.grace_days.to_string(),
        name: item.name.clone(),
        host: item.host.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_federation_cycle_views(items: &[StrictFederationCycle]) -> Vec<StrictFederationCycleView> {
    let mut views: Vec<StrictFederationCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_federation_cycle_view(item));
    }
    views
}

pub fn sum_strict_federation_cycle_view(view: &StrictFederationCycleView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_grace_days as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_federation_cycle("code-332", "title-948");
        let view = to_strict_federation_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_federation_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
