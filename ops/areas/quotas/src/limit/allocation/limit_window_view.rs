use super::*;

use acme_kit::text;

/// How screens and exports show a limit window.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitWindowView {
    pub code: String,
    pub name: String,
    pub visible: String,
    pub host: String,
    pub channel_text: String,
    pub timezone: String,
    pub batch_size: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_limit_window_view(item: &LimitWindow) -> LimitWindowView {
    LimitWindowView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        visible: format!("{}", item.visible),
        host: text::truncate(&item.host, 120),
        channel_text: item.channel.clone(),
        timezone: text::truncate(&item.timezone, 40),
        batch_size: item.batch_size.to_string(),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_window_views(items: &[LimitWindow]) -> Vec<LimitWindowView> {
    let mut views: Vec<LimitWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_window_view(item));
    }
    views
}

pub fn sum_limit_window_view(view: &LimitWindowView) -> Result<f64, String> {
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
        let item = new_limit_window("code-848", "name-796");
        let view = to_limit_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
