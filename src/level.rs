extern crate nalgebra as na;
extern crate nphysics as np;
use nphysics3d::object::{DefaultBodySet, DefaultColliderSet};
use nphysics3d::force_generator::DefaultForceGeneratorSet;
use nphysics3d::joint::DefaultJointConstraintSet;
use nphysics3d::world::{DefaultMechanicalWorld, DefaultGeometricalWorld};

use crate::mazegen::Maze;

pub struct MazeRes {
    pub maze: Maze,
}

impl MazeRes {

    pub fn new() -> Self {
        let mut maze = Maze::new();
        let mut res = MazeRes {
            maze: maze,
        }

        res.gen_maze();

        return res;
    }

    pub fn gen_maze(&mut self) {
        self.maze.reset();
        self.maze.build();
    }
}

