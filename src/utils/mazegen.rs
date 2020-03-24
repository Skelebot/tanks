use rand::{thread_rng, Rng};
 
#[derive(Clone, Copy)]
pub struct Cell {
    pub col: usize,
    pub row: usize,
}
 
impl Cell {
    fn from(col: usize, row: usize) -> Cell {
        Cell {col, row}
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::from(0, 0)
    }
}
 
pub struct Maze {
    pub width: usize,
    pub height: usize,
    cells: Vec<Vec<bool>>,  //Cell visited/unvisisted
    pub walls_h: Vec<Vec<bool>>,   //horizontal walls existing/removed
    pub walls_v: Vec<Vec<bool>>,   //vertical walls existing/removed
    pub start_cell: Cell,
    pub end_cell: Cell,
    //Thread_rng is not Send+Sync, so we couldn't use Maze in resources
    //thread_rng: ThreadRng,                  //Random numbers generator
}
 
impl Maze {
 
    ///Inits the maze, with all the cells unvisited and all the walls active
    pub fn new(width: usize, height: usize) -> Maze {
        Maze { 
            width: width,
            height: height,
            cells: vec![vec![true; height]; width], 
            walls_h: vec![vec![true; width]; height + 1],
            walls_v: vec![vec![true; width + 1]; height],
            start_cell: Cell::default(),
            end_cell: Cell::default()
        }
    }

    ///Reset the maze
    pub fn reset(&mut self) {
        self.cells = vec![vec![true; self.height]; self.width];
        self.walls_h = vec![vec![true; self.width]; self.height + 1];
        self.walls_v = vec![vec![true; self.width + 1]; self.height];
        self.start_cell = Cell::default();
        self.end_cell = Cell::default();
    }
 
    ///(Randomly) chooses the starting cell
    fn first(&mut self) -> Cell {
        let mut thread_rng = thread_rng();
        let first = Cell::from(thread_rng.gen_range(0, self.width), thread_rng.gen_range(0, self.height));
        self.start_cell = first; 
        return first;
    }
 
    ///Opens the enter and exit doors
    #[allow(unused)]
    pub fn open_doors(&mut self) {
        let mut thread_rng = thread_rng();
        let from_top: bool = thread_rng.gen();
        let limit = if from_top { self.width } else { self.height };
        let door = thread_rng.gen_range(0, limit);
        let exit = thread_rng.gen_range(0, limit);
        if from_top { 
            self.walls_h[0][door] = false;
            self.walls_h[self.height][exit] = false;
        } else {
            self.walls_v[door][0] = false;
            self.walls_v[exit][self.width] = false;
        }
    }
 
    ///Removes a wall between the two Cell arguments
    fn remove_wall(&mut self, cell1: &Cell, cell2: &Cell) {
        if cell1.row == cell2.row {
            self.walls_v[cell1.row][if cell1.col > cell2.col { cell1.col } else { cell2.col }] = false;
        } else { 
            self.walls_h[if cell1.row > cell2.row { cell1.row } else { cell2.row }][cell1.col] = false;
        };
    }
 
    ///Returns a random non-visited neighbor of the Cell passed as argument
    fn neighbor(&mut self, cell: &Cell) -> Option<Cell> {
        let mut thread_rng = thread_rng();
        self.cells[cell.col][cell.row] = false;
        let mut neighbors = Vec::new();
        if cell.col > 0 && self.cells[cell.col - 1][cell.row] { neighbors.push(Cell::from(cell.col - 1, cell.row)); }
        if cell.row > 0 && self.cells[cell.col][cell.row - 1] { neighbors.push(Cell::from(cell.col, cell.row - 1)); }
        if cell.col < self.width - 1 && self.cells[cell.col + 1][cell.row] { neighbors.push(Cell::from(cell.col + 1, cell.row)); }
        if cell.row < self.height - 1 && self.cells[cell.col][cell.row + 1] { neighbors.push(Cell::from(cell.col, cell.row + 1)); }
        if neighbors.is_empty() {
            None
        } else {
            let next = neighbors.get(thread_rng.gen_range(0, neighbors.len())).unwrap();
            self.remove_wall(cell, next);
            Some(*next)
        }
    }
 
    ///Builds the maze (runs the Depth-first search algorithm)
    pub fn build(&mut self) {
        let mut cell_stack: Vec<Cell> = Vec::new();
        let mut next = self.first();
        loop {
            while let Some(cell) = self.neighbor(&next) {
                cell_stack.push(cell);
                next = cell;
            }
            match cell_stack.pop() {
                Some(cell) => next = cell,
                None => {
                    self.end_cell = next;
                    break;
                },
            }
        }
    }
 
    ///Displays a wall
    fn paint_wall(h_wall: bool, active: bool) {
        if h_wall {
            print!("{}", if active { "+---" } else { "+   " });
        } else {
            print!("{}", if active { "|   " } else { "    " });
        }
    }
 
    ///Displays a final wall for a row
    fn paint_close_wall(h_wall: bool) {
        if h_wall { println!("+") } else { println!() }
    }
 
    ///Displays a whole row of walls
    fn paint_row(&self, h_walls: bool, index: usize) {
        let iter = if h_walls { self.walls_h[index].iter() } else { self.walls_v[index].iter() };
        for &wall in iter {
            Maze::paint_wall(h_walls, wall);
        }
        Maze::paint_close_wall(h_walls);
    } 
 
    ///Paints the maze
    #[allow(unused)]
    pub fn paint(&self) {
        for i in 0 .. self.width {
            self.paint_row(true, i);
            self.paint_row(false, i);
        }
        self.paint_row(true, self.width);
    }
}
