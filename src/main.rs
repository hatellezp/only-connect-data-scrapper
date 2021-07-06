use std::ops::Deref;

use std::process::Command;

use regex::Regex;
use scraper::{ElementRef, Html, Selector};

use select::document::Document;
use select::predicate::Name;

use structures::Game;

use crate::utilities::filter_link_for_episode;
use std::fs::File;
use std::io::Write;

mod data;
mod structures;
mod utilities;

const OCDB_EPISODES_ADDR: &str = "https://ocdb.cc/episodes/";

// const PATH_IN_TXT: &str = "data/game_maker/in.txt";

const PATH_IN_TXT: &str = "in.txt";
// const PATH_GAMES: &str = "game_data/";
// const PATH_INDEX_JS: &str = "index.js";
// const PATH_OUT_JSON: &str = "out.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // prepare the terrain
    let _result = std::fs::create_dir("game_data");

    // to be used later, when media clues fonction
    // let _result = std::fs::create_dir("data");
    // let _result = std::fs::create_dir("data");

    // this first part gather all the links
    // async call
    let resp_await = reqwest::get(OCDB_EPISODES_ADDR);

    // unbox from async call by await method
    let resp = resp_await.await?.text().await?;

    // create a vector for each episode link
    let mut links: Vec<&str> = Vec::new();

    // get the html code from the get request
    let doc = Document::from(resp.as_str());

    // extract hyperlinks from the document and put them in the links vector
    doc.find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .for_each(|x| {
            if x.len() > 25 {
                (&mut links).push(x)
            }
        });

    // filter non episode links
    let links = links
        .iter()
        .filter_map(|s| filter_link_for_episode(*s))
        .collect::<Vec<&str>>();
    // links are gathered and then processed on at the time

    let links_length = links.len();
    println!("there are {} links to be processed", links_length);

    for (index, link) in links.iter().enumerate() {
        let a = process_episode_link(*link);
        let _ = a.await?;

        println!("still {} links to be processed", links_length - index - 1);
    }

    Ok(())
}

async fn process_episode_link(link: &str) -> Result<(), Box<dyn std::error::Error>> {
    // answer to the selected link
    let resp = reqwest::get(link).await?.text().await?;

    // parse the document
    let document = Html::parse_document(&resp);

    // this function create the game name for the file storing
    let game_name = utilities::built_game_name(&document).unwrap();
    let episode = game_name.1;
    let series = game_name.0;
    let game_name = game_name.2;

    let div_selector = Selector::parse("div").unwrap();

    // create episode keeper
    let mut game = Game::new(&game_name, series, episode);

    let mut wall_counter: u16 = 0;

    for element in document.select(&div_selector) {
        if let Some("content") = element.value().attr("class") {
            let mut next_round: Option<u8> = None;

            for el in element.children() {
                // ignore pure espace text elements
                if el.value().is_text()
                    && utilities::remove_escapes_characters(el.value().as_text().unwrap().deref())
                        .is_empty()
                {
                    continue;
                }

                if el.value().is_element() {
                    let el_ref = ElementRef::wrap(el).unwrap();

                    let answer = utilities::element_is_h2_round(&el_ref);

                    if let Some((id, _name)) = &answer {
                        next_round = Some(*id);
                        // println!("this is a h2 title, {:?}", &answer);
                        continue;
                    }

                    if let Some(id) = next_round {
                        if utilities::element_is_div_question(&el_ref) {
                            // println!("round: {:?}", &el.value());

                            match id {
                                1 => {
                                    process_question1(&el_ref);
                                }
                                2 => {
                                    process_question2(&el_ref);
                                }
                                3 => {
                                    process_question3(&mut game, wall_counter, &el_ref);

                                    wall_counter += 1;
                                }
                                4 => {
                                    parse_question4(&el_ref);
                                }
                                _ => panic!("bad number for question type!!!"),
                            }
                        } else if utilities::get_element_tag(&el_ref) == "div"
                            && el_ref.value().attr("class").is_some()
                            && el_ref.value().attr("class").unwrap() == "vowel-round"
                        {
                            // println!("round: {:?}", &el.value());
                        }
                    }
                }
            }
        }
    }

    println!(" -- this game name is: {}", &game_name);

    // text for the wall question created
    let for_in_txt = game.create_txt_with_only_wall_question();

    /*
       - first step: write to file "in.txt"
       - second step: execute node "index.js" on "in.txt"
       - third step: copy "out.json" to "game_data/game_name.json"
    */

    // first step
    let mut in_txt_file = File::create(PATH_IN_TXT).unwrap();
    let _ = in_txt_file.write_all(for_in_txt.as_bytes());

    // second step
    let index_js_command = ["-c", "node index.js"];
    let output = Command::new("sh")
        .args(&index_js_command)
        .output()
        .expect("failed to execute command");
    let _result = output.stdout;

    // third step
    let mv_command_specific = format!("mv out.json game_data/{}.json", &game_name);
    let mv_command = ["-c", &mv_command_specific];

    let output = Command::new("sh")
        .args(&mv_command)
        .output()
        .expect("failed to execute command");
    let _result = output.stdout;

    Ok(())
}

pub fn process_question1(_el: &ElementRef) {}
pub fn process_question2(_el: &ElementRef) {}

pub fn process_question3(game: &mut Game, wall_number: u16, el: &ElementRef) {
    let re = Regex::new(r"\.*wall-container\.*").unwrap();

    for child in el.children() {
        if child.value().is_element() {
            let el_ref = ElementRef::wrap(child).unwrap();

            if utilities::get_element_tag(&el_ref) == "div"
                && el_ref.value().attr("class").is_some()
                && re.is_match(el_ref.value().attr("class").unwrap())
            {
                let mut div_counter: u16 = 0;
                let mut label_counter: u16 = 0;
                for inner_child in el_ref.children() {
                    if inner_child.value().is_text() {
                        continue;
                    }

                    if inner_child.value().is_element()
                        && utilities::get_element_tag(&ElementRef::wrap(inner_child).unwrap())
                            == "div"
                    {
                        let clue = utilities::clean_and_collect_element_text(
                            &ElementRef::wrap(inner_child).unwrap(),
                        );
                        game.walls[wall_number as usize].row[div_counter as usize] = clue.clone();

                        div_counter += 1;
                    }

                    if inner_child.value().is_element()
                        && utilities::get_element_tag(&ElementRef::wrap(inner_child).unwrap())
                            == "label"
                    {
                        let answer = utilities::clean_and_collect_element_text(
                            &ElementRef::wrap(inner_child).unwrap(),
                        );
                        game.walls[wall_number as usize].answer[label_counter as usize] =
                            answer.clone();

                        label_counter += 1;
                    }
                }
            }
        }
    }
}

pub fn parse_question4(_el: &ElementRef) {}
