use crate::data::{TXT_HEAD, TXT_TAIL};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Game {
    pub name: String,
    pub series: u8,
    pub episode: u8,
    pub walls: [Box<WallQuestion>; 2],
}

impl Game {
    pub fn new(name: &str, series: u8, episode: u8) -> Game {
        let w1 = WallQuestion::new();
        let w2 = WallQuestion::new();
        let wall1 = Box::new(w1);
        let wall2 = Box::new(w2);

        let walls = [wall1, wall2];

        Game {
            name: String::from(name),
            series,
            episode,
            walls,
        }
    }

    pub fn create_txt_wall_question(&self) -> String {
        let mut wall_text = String::from("!walls\n\n");

        for index in 0..2 {
            let inner_text = format!(
                "## WALL {}\n\n{}",
                index + 1,
                self.walls[index].create_txt_text()
            );

            wall_text.push_str(&inner_text);
        }

        wall_text
    }

    pub fn create_txt_with_only_wall_question(&self) -> String {
        let mut text = String::new();

        text.push_str(TXT_HEAD);
        text.push('\n');
        text.push_str(&self.create_txt_wall_question());
        text.push_str(TXT_TAIL);

        text
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WallQuestion {
    pub row: [String; 16],
    pub answer: [String; 4],
}

impl Default for WallQuestion {
    fn default() -> Self {
        WallQuestion::new()
    }
}

impl WallQuestion {
    pub fn new() -> WallQuestion {
        let row: [String; 16] = [
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ];
        let answer: [String; 4] = [String::new(), String::new(), String::new(), String::new()];

        WallQuestion { row, answer }
    }

    pub fn create_txt_text(&self) -> String {
        let mut wall_text = String::new();

        for index in 0..4 {
            let inner_index: usize = 4 * index;

            let inner_string = format!(
                "- {}\n{}\n{}\n{}\n{}\n\n",
                &self.answer[index],
                &self.row[inner_index],
                &self.row[inner_index + 1],
                &self.row[inner_index + 2],
                &self.row[inner_index + 3],
            );

            wall_text.push_str(&inner_string);
        }

        wall_text
    }
}

// pub struct ConnectionQuestion {}
// pub struct SequenceQuestion {}
// pub struct MissingVowelQuestion {}
