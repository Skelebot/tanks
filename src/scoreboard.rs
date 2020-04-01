use crate::tank::Team;
use amethyst::ecs::{Entity, WriteStorage};
use amethyst::ui::UiText;

/// Scoreboard resource that systems can use to read or write to the score counter
pub struct Scoreboard {
    scores: Vec<u32>,
    pub texts: Vec<Entity>,
}

impl Scoreboard {
    /// Creates a new Scoreboard; By default every team's score is 0
    ///
    /// Because we can't iter over enums in Rust, we can't determine
    /// the length of the Team enum, so this Vec declaration has to 
    /// be changed everytime we add a team or remove one
    pub fn new() -> Self {
        Scoreboard {
            scores: vec![0, 0],
            texts: vec![]
        }
    }
    /// Adds one to a team's score and changes the coresponding UI counter's text
    pub fn score(&mut self, team: Team, ui_text: &mut WriteStorage<UiText>) {
        self.scores[team as usize] += 1;
        ui_text.get_mut(self.get_text(team)).unwrap()
            .text = self.get_score(team).to_string();
    }
    /// Reads a score for a team
    pub fn get_score(&self, team: Team) -> u32 {
        self.scores[team as usize]
    }
    /// Returns an entity of a team's UI counter's text
    pub fn get_text(&self, team: Team) -> Entity {
        self.texts[team as usize]
    }
}
