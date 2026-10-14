use super::*;

use acme_kit::text;

/// How screens and exports show a token target.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenTargetView {
    pub code: String,
    pub name_text: String,
    pub maximum_text: String,
    pub channel_text: String,
    pub created_at: String,
    pub minimum: String,
    pub locale: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_token_target_view(item: &TokenTarget) -> TokenTargetView {
    TokenTargetView {
        code: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        channel_text: item.channel.clone(),
        created_at: item.created_at.to_string(),
        minimum: format!("{:.2}", item.minimum),
        locale: item.locale.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_target_views(items: &[TokenTarget]) -> Vec<TokenTargetView> {
    let mut views: Vec<TokenTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_target_view(item));
    }
    views
}

pub fn sum_token_target_view(view: &TokenTargetView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_maximum + (parsed_created_at as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_target("code-814", "name-721");
        let view = to_token_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
