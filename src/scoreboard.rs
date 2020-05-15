use crate::tank::Team;
use amethyst::ecs::{Entity, WriteStorage};
use amethyst::ui::UiText;

/// Scoreboard resource that systems can use to read or write to the score counter
pub struct Scoreboard {
    scores: Vec<u32>,
    alive: Vec<Team>,
    pub texts: Vec<Entity>,
}

impl Scoreboard {
    /// Creates a new Scoreboard; By default every team's score is 0

    /// Because we can't iter over enums in Rust, we can't determine
    /// the length of the Team enum, so this Vec declaration has to 
    /// be changed everytime we add a team or remove one. Oof.
    pub fn new() -> Self {
        Scoreboard {
            scores: vec![0, 0],
            alive: vec![Team::Red, Team::Blue],
            texts: vec![]
        }
    }
    /// Report that the tank was destroyed so we can determine the winner later
    pub fn report_destroyed(&mut self, team: Team) {
        self.alive.retain(|t| *t != team);
    }
    
    /// Check who is still alive (only one tank should be) and update it's score
    pub fn update_winners(&mut self, ui_text: &mut WriteStorage<UiText>) {
        // We could drain(..) here instead of clear() following iter(),
        // but we need access to other self fields so it's not possible
        for winner in self.alive.iter() {
            self.scores[*winner as usize] += 1;
            ui_text.get_mut(self.get_text(*winner)).unwrap()
                .text = self.get_score(*winner).to_string();
        }
        self.alive.clear();
        self.alive.push(Team::Red);
        self.alive.push(Team::Blue);
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
