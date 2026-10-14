use super::*;

use acme_kit::text;

/// How screens and exports show a silence group.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceGroupView {
    pub code: String,
    pub name: String,
    pub region: String,
    pub scope: String,
    pub cipher: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_silence_group_view(item: &SilenceGroup) -> SilenceGroupView {
    SilenceGroupView {
        code: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 80),
        region: text::truncate(&item.region, 40),
        scope: item.scope.clone(),
        cipher: item.cipher.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_group_views(items: &[SilenceGroup]) -> Vec<SilenceGroupView> {
    let mut views: Vec<SilenceGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_group_view(item));
    }
    views
}

pub fn sum_silence_group_view(view: &SilenceGroupView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_group("code-898", "name-954");
        let view = to_silence_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
