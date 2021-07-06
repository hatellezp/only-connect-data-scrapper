use regex::Regex;
use scraper::{ElementRef, Html, Selector};

pub fn clean_and_collect_element_text(el: &ElementRef) -> String {
    let t = el.text().collect::<Vec<&str>>();
    let t = t
        .iter()
        .map(|x| remove_escapes_characters(*x))
        .collect::<Vec<_>>();
    let t = t
        .iter()
        .filter_map(|x| purge_bad_strings(x))
        .collect::<Vec<_>>();

    t[0].clone()
}

pub fn purge_bad_strings(string: &str) -> Option<String> {
    let s = remove_escapes_characters(string);
    if s.is_empty() || &s == " " || &s == "Answer" || &s == "  " {
        None
    } else {
        Some(s)
    }
}

pub fn element_is_div_question(el: &ElementRef) -> bool {
    let op_str = el.value().attr("class");
    let tag = get_element_tag(el);

    match (op_str, tag) {
        (Some("question"), "div") => true,
        (_, _) => false,
    }
}

pub fn element_is_h2_round(el: &ElementRef) -> Option<(u8, String)> {
    let op_str = el.value().attr("id");
    let tag = get_element_tag(el);

    match (tag, op_str) {
        ("h2", Some(round_id)) => {
            let re = Regex::new(r"round(\d+)").unwrap();
            let mut number_id: u8 = 0;

            for cap in re.captures(round_id) {
                number_id = String::from(&cap[1]).parse::<u8>().unwrap();

                break;
            }

            let inner_text_op = get_element_text(el);

            // for a match, like an Option::map
            inner_text_op.map(|inner_text| (number_id, inner_text))
        }
        (_, _) => None,
    }
}

pub fn get_element_tag<'a>(el: &'a ElementRef) -> &'a str {
    el.value().name()
}

pub fn built_game_name(document: &Html) -> Option<(u8, u8, String)> {
    let h1_selector = Selector::parse("h1").unwrap();
    let h2_selector = Selector::parse("h2").unwrap();

    let mut series: u8 = 0;
    let mut episode: u8 = 0;
    let mut game_name = String::new();

    for h2 in document.select(&h2_selector) {
        if let Some("episode_meta") = h2.value().attr("class") {
            let op_text = get_element_text(&h2);

            if let Some(text) = op_text {
                let se_op = find_series_and_episode(&text);

                if let Some((s, e)) = se_op {
                    let header = format!("S{}_E{}", s, e);
                    game_name.push_str(&header);

                    series = s;
                    episode = e;

                    break;
                }
            }
        }
    }

    for h1 in document.select(&h1_selector) {
        if let Some("internaltext") = h1.value().attr("id") {
            let op_text = get_element_text(&h1);

            if let Some(s) = op_text {
                game_name.push('_');
                let episode_name = s.replace(" ", "_");
                game_name.push_str(&episode_name);

                break;
            }
        }
    }

    match game_name.len() {
        0 => None,
        _ => Some((series, episode, game_name)),
    }
}

pub fn find_series_and_episode(s: &str) -> Option<(u8, u8)> {
    let re = Regex::new(r"Series (\d+),\w*Episode (\d+)").unwrap();

    let mut series: Option<u8> = None;
    let mut episode: Option<u8> = None;

    for cap in re.captures_iter(s) {
        let s = String::from(&cap[1]).parse::<u8>().unwrap();
        let e = String::from(&cap[2]).parse::<u8>().unwrap();
        series = Some(s);
        episode = Some(e);
    }

    match (series, episode) {
        (Some(s), Some(e)) => Some((s, e)),
        (_, _) => None,
    }
}

pub fn remove_escapes_characters(s: &str) -> String {
    s.replace("\t", "").replace("\n", "").replace("\r", "")
}

pub fn get_element_text(el: &ElementRef) -> Option<String> {
    let op_text = el.children().next().unwrap().value().as_text();

    match op_text {
        Some(e) => {
            let a = String::from(e.clone().text);
            Some(remove_escapes_characters(&a))
        }
        _ => None,
    }
}

pub fn filter_link_for_episode(s: &str) -> Option<&str> {
    let escape = r"\u002F";
    let regex_text = format!(
        r"\.*https:{}{}ocdb.cc{}episode{}",
        escape, escape, escape, escape
    );
    let re = Regex::new(&regex_text).unwrap();

    if let true = re.is_match(s) {
        Some(s)
    } else {
        None
    }
}
