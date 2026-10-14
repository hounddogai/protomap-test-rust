use super::*;

use acme_kit::text;

/// How screens and exports show a release schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseScheduleView {
    pub code: String,
    pub label: String,
    pub batch_size: String,
    pub host_text: String,
    pub locale: String,
    pub channel_text: String,
    pub name_text: String,
    pub status_label: String,
}

pub fn to_release_schedule_view(item: &ReleaseSchedule) -> ReleaseScheduleView {
    ReleaseScheduleView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        batch_size: item.batch_size.to_string(),
        host_text: item.host.clone(),
        locale: text::truncate(&item.locale, 120),
        channel_text: item.channel.clone(),
        name_text: item.name.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_schedule_views(items: &[ReleaseSchedule]) -> Vec<ReleaseScheduleView> {
    let mut views: Vec<ReleaseScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_schedule_view(item));
    }
    views
}

pub fn sum_release_schedule_view(view: &ReleaseScheduleView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_schedule("code-242", "label-729");
        let view = to_release_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
