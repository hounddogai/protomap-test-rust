use super::*;

use acme_kit::text;

/// How screens and exports show a identity schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityScheduleView {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub country: String,
    pub created_at_text: String,
    pub status_label: String,
}

pub fn to_identity_schedule_view(item: &IdentitySchedule) -> IdentityScheduleView {
    IdentityScheduleView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        cipher: item.cipher.clone(),
        country: item.country.clone(),
        created_at_text: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_schedule_views(items: &[IdentitySchedule]) -> Vec<IdentityScheduleView> {
    let mut views: Vec<IdentityScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_schedule_view(item));
    }
    views
}

pub fn sum_identity_schedule_view(view: &IdentityScheduleView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_created_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_schedule("code-752", "name-672");
        let view = to_identity_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
