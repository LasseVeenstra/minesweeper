use termion::event::{Key, Event, MouseEvent, MouseButton};
use termion::input::{TermRead, MouseTerminal};
use termion::raw::IntoRawMode;
use std::io::{Write, stdout, stdin};
use rand::random;
use std::fmt;
use libm::exp;


fn shifted_sigmoid(x: i32) -> f64 {
    0.6 / (1.0 + exp(((-1*x) + 5) as f64))
}


#[derive(Clone, Debug)]
struct Cell{
    bomb: bool, // whether or not a bomb
    flagged: bool, // whether or not it has been flagged
    visable: bool,
    num_neighbours: u8,
}

impl Cell {
    fn new() -> Cell {
        Cell {
            bomb: false,
            flagged: false,
            visable: false,
            num_neighbours: 0
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.flagged {
            write!(f, "F")
        }
        else if !self.visable || (self.visable && self.bomb) {
            write!(f, "▒")
        }
        else if self.visable && self.num_neighbours == 0 {
            write!(f, " ")
        }
        else {
            write!(f, "{}", self.num_neighbours)
        }
    }
}

struct MineSweeper {
    width: u8,
    height: u8,
    field: Vec<Vec<Cell>>,
    field_origin: (u16, u16),
    moves: u16,
    level: u16,
    page: String
}

impl fmt::Display for MineSweeper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut res: String = String::new();
        for cell_vec in &self.field {
            res.push_str(String::from_utf8(vec![b' '; self.field_origin.0 as usize]).unwrap().as_str());
            for cell in cell_vec {
                res.push_str(cell.to_string().as_str());
                res.push_str(" ");
            }
            res.push_str("\n\r");
        }
        write!(f, "{}", res)
    }
}

impl MineSweeper {
    fn new(field_origin: (u16, u16)) -> MineSweeper {
        let (mut x, mut y) = termion::terminal_size().unwrap();
        x = (x / 2) - 1 - field_origin.0;
        y = y - field_origin.1 - 4;
        let mut field: Vec<Vec<Cell>> = Vec::new();
        for _ in 0..y+1 {
            let mut field_row: Vec<Cell> = Vec::new();
            for _ in 0..x+1 {
                field_row.push(Cell::new());
            }
            field.push(field_row);
        }
        let mut ms = MineSweeper {width: x as u8, height: y as u8, field, field_origin,
             moves: 0, level:1, page: String::from("Homescreen")};
        ms.update_neighbours();
        ms
    }

    fn place_bombs(&mut self, bomb_prob: f64) {
        for cell_vec in &mut self.field {
            for cell in cell_vec {
                let x: f64 = random::<f64>();
                cell.bomb = if x < bomb_prob {true} else {false};
            }
        }
    }

    fn reset(&mut self) {
        for cell_vec in &mut self.field {
            for cell in cell_vec {
                cell.bomb = false;
                cell.num_neighbours = 0;
                cell.visable = false;
                cell.flagged = false;
            }
        }
        self.moves = 0;
    }

    fn terminalxy2fieldxy(&mut self, x :u16, y :u16) -> (i32, i32) {
        let x_hat: i32 = ((x as i32 - self.field_origin.0 as i32) as f32 / 2.0).round() as i32 - 1;
        let y_hat: i32 = y as i32  - self.field_origin.1 as i32;
        (x_hat, y_hat)
    }

    fn flag(&mut self, x: usize, y: usize) {
        self.field[y][x].flagged = !self.field[y][x].flagged;
    }

    fn update_neighbours (&mut self) {
        for x in 0..(self.width+1) as usize {
            for y in 0..(self.height+1) as usize {
                if y >= 1 {
                    self.field[y][x].num_neighbours += self.field[y-1][x].bomb as u8;
                    if x >= 1 {
                        self.field[y][x].num_neighbours += self.field[y-1][x-1].bomb as u8;
                    }
                    if x < self.width as usize {
                        self.field[y][x].num_neighbours += self.field[y-1][x+1].bomb as u8;
                    }
                }
                if y < self.height as usize {
                    self.field[y][x].num_neighbours += self.field[y+1][x].bomb as u8;
                    if x >= 1 {
                        self.field[y][x].num_neighbours += self.field[y+1][x-1].bomb as u8;
                    }
                    if x < self.width as usize {
                        self.field[y][x].num_neighbours += self.field[y+1][x+1].bomb as u8;
                    }
                }
                if x >= 1 {
                    self.field[y][x].num_neighbours += self.field[y][x-1].bomb as u8;
                }
                if x < self.width as usize {
                    self.field[y][x].num_neighbours += self.field[y][x+1].bomb as u8;
                }
            }

        }
    }

    fn open(&mut self, x: usize, y: usize) {
        self.field[y][x].visable = true;
        let x = x as i32;
        let y = y as i32;
        // implement the recursion part, where neighbours are also opened
        let list = [(x-1, y-1), (x-1, y), (x-1, y+1), (x, y-1), (x, y+1),
         (x+1, y-1), (x+1, y), (x+1, y+1)];
        for (a, b) in list {
            if a >= 0 && a <= self.width as i32 && b >= 0 && b <= self.height as i32{
                if self.field[b as usize][a as usize].num_neighbours == 0 && 
                 !self.field[b as usize][a as usize].visable {
                    self.open(a as usize, b as usize);
                }
                self.field[b as usize][a as usize].visable = true;
            }
        }
    }

    fn display_field(& self, stdout: &mut MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>) {
        write!(stdout, "{}{}", termion::cursor::Goto(1, self.field_origin.1), self).unwrap();
        stdout.flush().unwrap();
    }

    fn make_move(&mut self, event: MouseEvent) {
        match event {
            MouseEvent::Press(presstype, x, y) => {
                let (x_hat, y_hat) = self.terminalxy2fieldxy(x, y);

                if x_hat >= 0 && y_hat >= 0 && x_hat <= self.width as i32 && y_hat <= self.height as i32 {
                    if self.moves == 0 {
                        self.place_bombs(shifted_sigmoid(self.level as i32));
                        self.field[y_hat as usize][x_hat as usize].bomb = false;
                        self.update_neighbours();
                    }
                    match presstype {
                        MouseButton::Right => self.flag(x_hat as usize, y_hat as usize),
                        MouseButton::Left => {        
                            // if we try to open a flagged cell
                            if self.field[y_hat as usize][x_hat as usize].flagged {
                                return;
                            }
                            // if we hit a bomb
                            else if self.field[y_hat as usize][x_hat as usize].bomb {
                                self.page = String::from("Gameover");
                                return;
                            }
                            // if we try to open a cell that is already visible
                            else if self.field[y_hat as usize][x_hat as usize].visable {
                                return;
                            }
                            else {
                                self.open(x_hat as usize, y_hat as usize);
                                self.moves += 1;
                            }},
                        _ => ()
                    }
                }
            },
            _ => (),
        }
    }

    fn gamescreen(&mut self, evt: Event) {
        match evt {
            Event::Key(Key::Char('q')) => {self.page = String::from("Homescreen"); self.reset()},
            Event::Key(Key::Char('r')) => self.reset(),
            Event::Mouse(me) => self.make_move(me),
            _ => {}
        }
    }

    fn write_gamescreen(&self, stdout: &mut MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>) {
        write!(stdout, "{}{}\n\n
        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+\r
        | difficulty level: {} | move: {} | r: reset game | q: quit |\r
        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+", termion::clear::All, termion::cursor::Goto(1, 1),
            self.level, self.moves).unwrap();
        self.display_field(stdout);
        write!(stdout, "\n\r Left click to dig up a spot, right click to place a flag!").unwrap();
    }

    fn homescreen(&mut self, evt: Event) {
        match evt {
            Event::Key(Key::Char('q')) => self.page = String::from("Quit"),
            Event::Key(Key::Char('p')) => self.page = String::from("Gamescreen"),
            // regardless of having pressed shift we want to upgrade the level
            Event::Key(Key::Char('=')) => {if self.level < 9 {self.level += 1}},
            Event::Key(Key::Char('+')) => {if self.level < 9 {self.level += 1}},

            Event::Key(Key::Char('-')) => {if self.level > 1 {self.level -= 1}},
            _ => {}
        }
    }

    fn write_homescreen(&self, stdout: &mut MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>) {
        let space = String::from_utf8(vec![b' '; (self.width/4) as usize]).unwrap();
        write!(stdout, "{}{}\n\n\r
        {}+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+\r
        {}| M | I | N | E | S | W | E | E | P | E | R |\r
        {}+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+\n\r\n\n\n\n
        +-+-+-+-+\r
        |  MENU |\r
        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+\r
        | p: play!  | q: quit game  |  +/- : change difficulty level (currently {})  |\r
        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+", 
        termion::clear::All, termion::cursor::Goto(1, 1), space, space, space, self.level).unwrap();
    }

    fn gameoverscreen(&mut self, evt: Event) {
        match evt {
            Event::Key(Key::Char('r')) => self.page = String::from("Homescreen"),
            _ => {}
        }
    }

    fn write_gameoverscreen(&mut self, stdout: &mut MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>) {
        write!(stdout, "{}{}\n\n
        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+\r
        | G | A | M | E | O | V | E | R |  r: return to menu  |\r
        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+", termion::clear::All, termion::cursor::Goto(1, 1)).unwrap();
        self.display_field(stdout);
    }

    fn run(&mut self) {
        let stdin = stdin();
        let mut stdout:MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>> = 
            MouseTerminal::from(stdout().into_raw_mode().unwrap());
        write!(stdout, "{}{}{}\n\n\nWelcome to MineSweeper! \n\n\n\rPress any button to start playing!", termion::clear::All,
         termion::cursor::Goto(1,1), termion::cursor::Hide).unwrap();
         stdout.flush().unwrap();
    
        for c in stdin.events() {
            let evt = c.unwrap();
            
            // process the event
            match self.page.as_str() {
                "Homescreen" => self.homescreen(evt),
                "Gamescreen" => self.gamescreen(evt),
                "Gameover" => self.gameoverscreen(evt),
                _ => {}
            }

            // update the screen, note that self.page might have been changed due to to previous match
            match self.page.as_str() {
                "Homescreen" => self.write_homescreen(&mut stdout),
                "Gamescreen" => self.write_gamescreen(&mut stdout),
                "Gameover" => self.write_gameoverscreen(&mut stdout),
                "Quit" => break,
                _ => {}
            }
            stdout.flush().unwrap();
        }
    }
}


fn main() {
    let mut ms: MineSweeper = MineSweeper::new((8, 9));
    ms.run();


}
