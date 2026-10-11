use fastrand;
use std::{io::{self, Write}, time::Duration, thread, sync::{Arc, Condvar, Mutex}};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::{
    cursor::{self, MoveTo},
    execute,
    queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen,Clear, ClearType,enable_raw_mode, disable_raw_mode},
};
use std::fmt;

fn main() {
    play_tetris();
}


fn play_tetris() -> io::Result<()> {
    let debug_mode = false;
    let level = Arc::new(Mutex::new(7u32)); 
    let worker_level = Arc::clone(&level);
    let board = Arc::new(Mutex::new(Board::new()));
    let worker_board = Arc::clone(&board);
    let mut first_tetromino = generate_random_tetromino(7);
    let tetromino_being_played = u8_to_tetromino(first_tetromino);
    let active_tetromino = Arc::new((Mutex::new(Some(tetromino_being_played)), Condvar::new()));
    let worker_active_tetromino = Arc::clone(&active_tetromino);
    thread::spawn(move || {
            let speeds = [798, 715, 632, 549, 466, 383, 300, 216, 133, 100, 83,83,83, 67, 67, 67, 50, 50, 50, 33, 33, 33, 33, 33, 33, 33, 33, 33, 33, 17];
            loop {
                //println!("moving block one down");
                let level = worker_level.lock().unwrap();
                let speed = if *level > 29 {
                    17
                } else {
                    speeds[*level as usize]
                };
                drop(level);
                thread::sleep(Duration::from_millis(speed));
                let (mutex, condvar) = &*worker_active_tetromino;
                let mut guard = mutex.lock().unwrap();
                while guard.is_none() {
                    //println!("guard is none waiting");
                    guard = condvar.wait(guard).unwrap();
                }
                let old_value = guard.take().expect("value should be present");
                
                let new_value = Tetromino::move_down(&old_value);
                let mut board = worker_board.lock().unwrap();

                //println!("{:?}",&new_value);
                if possible(&board, &new_value){
                    *guard = Some(new_value);
                } else {
                    if connected(&board, &old_value) {
                        //println!("adding block to board");
                        //println!("{:?}",old_value);
                        (*board).add_tetromino(&old_value);
                        *guard = None;
                        //println!("{:?}", &board);
                    } else {
                        *guard = Some(old_value);
                    }
                } 
            };
    });




    if !debug_mode {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
    }
    let points = [0, 40, 100, 300, 1200];
    let mut second_tetromino = generate_random_tetromino(first_tetromino);
    let mut score = 0;
    let mut lines = 0;
    loop {
        let (mutex, condvar) = &*active_tetromino;
        let mut guard = mutex.lock().unwrap();
        if guard.is_none() {
            //println!("master guard is none");
            *guard = Some(u8_to_tetromino(second_tetromino));
            first_tetromino = second_tetromino;
            second_tetromino = generate_random_tetromino(second_tetromino);
            let mut board = board.lock().unwrap();
            let rows_removed = (*board).remove_full_rowes();
            lines += rows_removed;
            let mut level = level.lock().unwrap();
            if lines > 10 {
                *level += 1;
                lines = lines % 10;
            }
            score += points[rows_removed as usize] * (*level + 1);
            //println!("notifing");
            condvar.notify_one();
            if board.board[board.board.len() - 1].iter().any(|&x| x != 0) {
                break;
            }
        }
        //println!("get board to print");
        let mut board = board.lock().unwrap();
        let level = level.lock().unwrap();
        if !debug_mode {render_game(&board, guard.as_ref().unwrap(),  second_tetromino, score, *level).unwrap();}
        let action = if event::poll(Duration::from_millis(0)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('r') => 4,
                        KeyCode::Char(' ') => 0,
                        KeyCode::Down =>  1,
                        KeyCode::Left => 2,
                        KeyCode::Right => 3,
                        KeyCode::Char('e') => 4,
                        KeyCode::Char('q') => 5,
                        KeyCode::Char('z') => break,
                        _ => continue,
                    }
                } else {-1}
            } else {-1}
        } else {-1};
        match action {
            -1 => continue,
            0 => {hard_drop_naive(&mut board, &guard.take().expect("should never happen"));
                  *guard = None;},
            1 => {let old_val = guard.take().expect("should never happen");
                  let new_val = Tetromino::move_down(&old_val);
                  if possible(&board, &new_val){
                      if connected(&board, &new_val) {
                          //println!("adding block to board");
                          (*board).add_tetromino(&new_val);
                          *guard = None;
                          //println!("{:?}", &board);
                      } else {
                          *guard = Some(new_val);
                      }
                  } else {
                      *guard = Some(old_val);
                  }                  
            },
            2 => {
                let old_val = guard.take().expect("should never happen");
                let new_val = Tetromino::move_left(&old_val);
                if possible(&board, &new_val) {
                    *guard = Some(new_val);
                } else {
                    *guard = Some(old_val);
                }
            }
            3 => {
                let old_val = guard.take().expect("should never happen");
                let new_val = Tetromino::move_right(&old_val);
                if possible(&board, &new_val) {
                    *guard = Some(new_val);
                } else {
                    *guard = Some(old_val);
                }
            }

            4 => {
                let old_val = guard.take().expect("should never happen");
                let new_val = Tetromino::rotate_right(&old_val);
                if possible(&board, &new_val) {
                    *guard = Some(new_val);
                } else {
                    *guard = Some(old_val);
                }
            }
            5 => {
                let old_val = guard.take().expect("should never happen");
                let new_val = Tetromino::rotate_left(&old_val);
                if possible(&board, &new_val) {
                    *guard = Some(new_val);
                } else {
                    *guard = Some(old_val);
                }
            }

            _ => panic!("should never happen")
        }
    }
    if !debug_mode {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, cursor::Show)?;
    }
    Ok(())
}



#[derive(Debug)]
struct Board {
board: [[u8 ; 10];20],
}


impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
     for row in self.board.iter().rev() {
            write!(f, "\u{2588}")?;
            for cell in row {
                if *cell != 0 {
                    write!(f,"\u{2588}\u{2588}")?
                } else {
                    write!(f, ". ")?;
                }

            }
            write!(f,"\u{2588}\r\n").unwrap();
        }
        write!(f, "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\r\n")?;
        Ok(())
        }
}


/// coordinates are from left to right top to bottom of default positin
/// * *   *     * *    * *          *       *  *
/// * +   +   * +        + *      * + *     +  +
///       *                               * *  * *
///       *
/// the plus is the rotation point
#[derive(Debug, Copy, Clone)]
enum Tetromino {
O(u8, (u8,u8),(u8,u8),(u8,u8),(u8,u8)),
I(u8, (u8,u8),(u8,u8),(u8,u8),(u8,u8)),
S(u8, (u8,u8),(u8,u8),(u8,u8),(u8,u8)),
Z(u8, (u8,u8),(u8,u8),(u8,u8),(u8,u8)),
T(u8, (u8,u8),(u8,u8),(u8,u8),(u8,u8)),
J(u8, (u8,u8),(u8,u8),(u8,u8),(u8,u8)),
L(u8, (u8,u8),(u8,u8),(u8,u8),(u8,u8)),
}

impl Tetromino {
    fn rotate_right(&self) -> Tetromino {
        match *self {
            Tetromino::O(rotation, c1,c2,c3,c4) => {Tetromino::O_rotate_right(rotation, c1, c2, c3, c4)}
            Tetromino::I(rotation, c1,c2,c3,c4) => {Tetromino::I_rotate_right(rotation, c1, c2, c3, c4)}
            Tetromino::S(rotation, c1,c2,c3,c4) => {Tetromino::S_rotate_right(rotation, c1, c2, c3, c4)}
            Tetromino::Z(rotation, c1,c2,c3,c4) => {Tetromino::Z_rotate_right(rotation, c1, c2, c3, c4)}
            Tetromino::T(rotation, c1,c2,c3,c4) => {Tetromino::T_rotate_right(rotation, c1, c2, c3, c4)}
            Tetromino::J(rotation, c1,c2,c3,c4) => {Tetromino::J_rotate_right(rotation, c1, c2, c3, c4)}
            Tetromino::L(rotation, c1,c2,c3,c4) => {Tetromino::L_rotate_right(rotation, c1, c2, c3, c4)}


        }

    }

    fn rotate_left(&self) -> Tetromino {
    self.rotate_right().rotate_right().rotate_right()
    }

    fn O_rotate_right(rotation:u8, (x1, y1):(u8,u8), (x2, y2):(u8,u8), (x3, y3):(u8,u8), (x4, y4):(u8,u8)) -> Tetromino{
        match rotation {
            0 => {Tetromino::O(1,(x4, y4 + 1),(x4 + 1, y4 + 1) ,(x4,y4),(x4 + 1, y4))},

            1 => {Tetromino::O(2, (x3, y3), (x3+1, y3), (x3, y3 - 1),(x3 + 1, y3 - 1))},
            2 => {Tetromino::O(3, (x1 - 1, y1), (x1, y1), (x1 - 1, y1 - 1), (x1, y1 - 1))},
            3 => {Tetromino::O(0, (x2 - 1, y2 + 1), (x2, y2 + 1), (x2 - 1, y2), (x2, y2))},
            _ => panic!("error: rotation should never be a value higher than 3")
        }
    }

    fn I_rotate_right(rotation:u8, (x1, y1):(u8,u8), (x2, y2):(u8,u8), (x3, y3):(u8,u8), (x4, y4):(u8,u8)) -> Tetromino {
        match rotation {
            0 => {if x2 < 2 {return Tetromino::I(rotation, (x1,y1), (x2,y2), (x3,y3), (x4,y4))}
                Tetromino::I(1, (x2-2, y2), (x2 -1 , y2), (x2, y2), (x2 + 1, y2))},
            1 => {Tetromino::I(2, (x3, y3 + 2), (x3, y3 + 1), (x3, y3), (x3, y3 - 1))},
            2 => {Tetromino::I(3, (x3- 1, y3), (x3, y3), (x3 + 1, y3), (x3 + 2, y3))},
            3 => {if y2 < 2 {return Tetromino::I(rotation, (x1,y1), (x2,y2), (x3,y3), (x4,y4))}
                Tetromino::I(0, (x2, y2 + 1), (x2, y2), (x2, y2 - 1), (x2, y2 - 2))},
            _ => panic!("error: rotation should never be a value higher than 3")
        }
    }

    fn S_rotate_right (rotation:u8, (x1, y1):(u8,u8), (x2, y2):(u8,u8), (x3, y3):(u8,u8), (x4, y4):(u8,u8)) -> Tetromino {
        match rotation {
            0 => {Tetromino::S(1, (x4, y4 + 1), (x4, y4), (x4 + 1, y4), (x4 + 1, y4 -1))},
            1 => {Tetromino::S(2, (x2, y2), (x2 + 1, y2), (x2 -1 , y2 - 1), (x2, y2 - 1))},
            2 => {Tetromino::S(3, (x1 - 1, y1 + 1), (x1 -1 , y1), (x1, y1), (x1, y1 - 1))},
            3 => {Tetromino::S(0, (x3, y3 + 1), (x3 + 1, y3 + 1), (x3 - 1, y3), (x3, y3))},
            _ => panic!("error: rotation should never be a value higher than 3")
        }
    }

    fn Z_rotate_right (rotation:u8, (x1, y1):(u8,u8), (x2, y2):(u8,u8), (x3, y3):(u8,u8), (x4, y4):(u8,u8)) -> Tetromino {
        match rotation {
            0 => {Tetromino::Z(1, (x3 + 1, y3 + 1), (x3, y3), (x3 + 1, y3), (x3, y3 -1))},
            1 => {Tetromino::Z(2, (x2 -1, y2), (x2, y2), (x2, y2 - 1), (x2 + 1, y2 - 1))},
            2 => {Tetromino::Z(3, (x2, y2 + 1), (x2 -1, y2), (x2, y2), (x2 -1 , y2 -1))},
            3 => {Tetromino::Z(0, (x3 - 1, y3 + 1), (x3, y3 + 1), (x3, y3), (x3 + 1, y3))},
            _ => panic!("error: rotation should never be a value higher than 3")

        }
    }

    fn T_rotate_right(rotation:u8, (x1, y1):(u8,u8), (x2, y2):(u8,u8), (x3, y3):(u8,u8), (x4, y4):(u8,u8)) -> Tetromino {
        match rotation {
            0 => {Tetromino::T(1, (x3, y3 + 1), (x3, y3), (x3 + 1, y3), (x3, y3 - 1))},
            1 => {Tetromino::T(2 , (x2 -1, y2), (x2, y2), (x2 + 1, y2), (x2, y2 - 1))},
            2 => {Tetromino::T(3, (x2, y2 + 1), (x2 - 1, y2), (x2, y2), (x2, y2 - 1))},
            3 => {Tetromino::T(0, (x3, y3 + 1), (x3 - 1, y3), (x3, y3), (x3 + 1, y3))},
            _ => panic!("error: rotation should never be a value higher than 3")
        }
    }

    fn J_rotate_right (rotation:u8, (x1, y1):(u8,u8), (x2, y2):(u8,u8), (x3, y3):(u8,u8), (x4, y4):(u8,u8)) -> Tetromino {
        match rotation {
            0 => {Tetromino::J(1, (x2 - 1, y2 + 1), (x2 -1 , y2), (x2, y2), (x2 + 1, y2))},
            1 => {Tetromino::J(2, (x3, y3 + 1), (x3 + 1, y3 + 1), (x3, y3), (x3, y3 - 1))},
            2 => {Tetromino::J(3, (x3 - 1, y3), (x3, y3), (x3 + 1, y3), (x3 + 1, y3 - 1))},
            3 => {Tetromino::J(0, (x2, y2 + 1), (x2, y2), (x2 - 1, y2 -1), (x2, y2 - 1))},
            _ => panic!("error: rotation should never be a value higher than 3")
        }
    }

    fn L_rotate_right (rotation:u8, (x1, y1):(u8,u8), (x2, y2):(u8,u8), (x3, y3):(u8,u8), (x4, y4):(u8,u8)) -> Tetromino {
        match rotation {
            0 => {Tetromino::L(1, (x2 - 1, y2), (x2, y2), (x2 + 1, y2), (x2 - 1, y2 - 1))},
            1 => {Tetromino::L(2, (x2 - 1, y2 + 1), (x2, y2 + 1), (x2, y2), (x2 , y2 - 1))},
            2 => {Tetromino::L(3, (x3 + 1, y3 + 1), (x3, y3), (x3 - 1, y3), (x3 + 1, y3))},
            3 => {Tetromino::L(0, (x2, y2 + 1), (x2, y2), (x2, y2 - 1), (x2 + 1, y2 - 1))},
            _ => panic!("error: rotation should never be a value higher than 3")
        }
    }







    fn move_right(&self) -> Tetromino {

        fn move_tetromino_right (rotation:u8, (x1, y1):(u8,u8), (x2, y2):(u8,u8), (x3, y3):(u8,u8), (x4, y4):(u8,u8)) -> (u8, (u8,u8), (u8,u8), (u8,u8), (u8,u8)) {
                (rotation, (x1 + 1, y1), (x2 + 1, y2), (x3 + 1, y3), (x4 + 1, y4))
            }

        match *self {
            Tetromino::O(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_right(rotation, c1,c2,c3,c4);
                                                    Tetromino::O(r, cn1, cn2, cn3, cn4)},
            Tetromino::I(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_right(rotation, c1, c2, c3, c4);
                                                    Tetromino::I(r, cn1, cn2, cn3, cn4)},
            Tetromino::S(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_right(rotation, c1, c2, c3, c4);
                                                    Tetromino::S(r, cn1, cn2, cn3, cn4)},
            Tetromino::Z(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_right(rotation, c1, c2, c3, c4);
                                                    Tetromino::Z(r, cn1, cn2, cn3, cn4)},
            Tetromino::T(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_right(rotation, c1, c2, c3, c4);
                                                    Tetromino::T(r, cn1, cn2, cn3, cn4)},
            Tetromino::J(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_right(rotation, c1, c2, c3, c4);
                                                    Tetromino::J(r, cn1, cn2, cn3, cn4)},
            Tetromino::L(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_right(rotation, c1, c2, c3, c4);
                                                    Tetromino::L(r, cn1, cn2, cn3, cn4)}
        }
        
    }

    fn move_left(&self) -> Tetromino {

        fn move_tetromino_left (rotation:u8, (x1, y1):(u8,u8), (x2, y2):(u8,u8), (x3, y3):(u8,u8), (x4, y4):(u8,u8)) -> (u8, (u8,u8), (u8,u8), (u8,u8), (u8,u8)) {
                (rotation, (x1 - 1, y1), (x2 - 1, y2), (x3 - 1, y3), (x4 - 1, y4))
            }

        match *self {
            Tetromino::O(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_left(rotation, c1,c2,c3,c4);
                                                    Tetromino::O(r, cn1, cn2, cn3, cn4)},
            Tetromino::I(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_left(rotation, c1, c2, c3, c4);
                                                    Tetromino::I(r, cn1, cn2, cn3, cn4)},
            Tetromino::S(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_left(rotation, c1, c2, c3, c4);
                                                    Tetromino::S(r, cn1, cn2, cn3, cn4)},
            Tetromino::Z(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_left(rotation, c1, c2, c3, c4);
                                                    Tetromino::Z(r, cn1, cn2, cn3, cn4)},
            Tetromino::T(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_left(rotation, c1, c2, c3, c4);
                                                    Tetromino::T(r, cn1, cn2, cn3, cn4)},
            Tetromino::J(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_left(rotation, c1, c2, c3, c4);
                                                    Tetromino::J(r, cn1, cn2, cn3, cn4)},
            Tetromino::L(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_left(rotation, c1, c2, c3, c4);
                                                    Tetromino::L(r, cn1, cn2, cn3, cn4)}
        }
        
    }

    fn move_down(&self) -> Tetromino {

        fn move_tetromino_down (rotation:u8, (x1, y1):(u8,u8), (x2, y2):(u8,u8), (x3, y3):(u8,u8), (x4, y4):(u8,u8)) -> (u8, (u8,u8), (u8,u8), (u8,u8), (u8,u8)) {
                (rotation, (x1, y1 - 1), (x2, y2- 1), (x3, y3 - 1), (x4, y4- 1))
            }

        match *self {
            Tetromino::O(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_down(rotation, c1,c2,c3,c4);
                                                    Tetromino::O(r, cn1, cn2, cn3, cn4)},
            Tetromino::I(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_down(rotation, c1, c2, c3, c4);
                                                    Tetromino::I(r, cn1, cn2, cn3, cn4)},
            Tetromino::S(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_down(rotation, c1, c2, c3, c4);
                                                    Tetromino::S(r, cn1, cn2, cn3, cn4)},
            Tetromino::Z(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_down(rotation, c1, c2, c3, c4);
                                                    Tetromino::Z(r, cn1, cn2, cn3, cn4)},
            Tetromino::T(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_down(rotation, c1, c2, c3, c4);
                                                    Tetromino::T(r, cn1, cn2, cn3, cn4)},
            Tetromino::J(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_down(rotation, c1, c2, c3, c4);
                                                    Tetromino::J(r, cn1, cn2, cn3, cn4)},
            Tetromino::L(rotation, c1,c2,c3,c4) => {let (r, cn1, cn2, cn3, cn4) = move_tetromino_down(rotation, c1, c2, c3, c4);
                                                    Tetromino::L(r, cn1, cn2, cn3, cn4)}
        }
    }

}



impl Board {
    fn new() -> Board {
        Board{board: [[0;10]; 20]}
    }

    fn add_tetromino(&mut self, tetromino:&Tetromino) {
     let ((x1, y1), (x2, y2), (x3, y3), (x4, y4)) = match tetromino {
        Tetromino::O(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::I(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::S(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::Z(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::T(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::J(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::L(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4)) => ((x1-1, y1-1), (x2-1, y2-1), (x3-1, y3-1), (x4-1, y4-1))
    };
    self.board[y1 as usize][x1 as usize] = 1;
    self.board[y2 as usize][x2 as usize] = 1;
    self.board[y3 as usize][x3 as usize] = 1;
    self.board[y4 as usize][x4 as usize]= 1;
    //println!("{} {} {} {} {} {} {} {}", x1, y1, x2, y2 , x3, y3 , x4, y4);
    //println!("{}",self)
}


    fn remove_full_rowes(&mut self) -> u32 {
        let mut write = 0;
        let mut removed = 0;

        // Keep non-full rows, preserving their order from bottom to top.
        for read in 0..self.board.len() {
            if self.board[read].iter().all(|&cell| cell != 0) {
                removed += 1;
            } else {
                self.board[write] = self.board[read];
                write += 1;
            }
        }

        // Clear the remaining rows at the top.
        for row in &mut self.board[write..] {
            *row = [0; 10];
        }

        removed
    }
}

fn possible(board:&Board, value:&Tetromino) -> bool {
    let ((x1, y1), (x2, y2), (x3, y3), (x4, y4)) = match value {
        Tetromino::O(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::I(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::S(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::Z(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::T(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::J(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::L(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4)) => ((*x1, *y1), (*x2, *y2), (*x3, *y3), (*x4, *y4))
    };
    if x1 < 1 || x2 < 1 || x3 < 1 || x4 < 1 || x1 > 10 || x2 > 10 || x3 > 10 || x4 > 10 || y1 < 1 || y2 < 1 || y3 < 1 || y4 < 1 || y1 > 20 || y2 > 20 || y3 > 20 || y4 > 20 {
        return false;
    }
    if board.board[(y1 -1) as usize][(x1 -1) as usize] == 0 && board.board[(y2 -1) as usize][(x2 - 1) as usize] == 0 && board.board[(y3 -1) as usize][(x3 -1) as usize] == 0 && board.board[(y4 - 1) as usize][(x4 -1) as usize] == 0 {
        return true;
    }
    false
}

fn connected(board: &Board, value:&Tetromino) -> bool {
    let ((x1, y1), (x2, y2), (x3, y3), (x4, y4)) = match value {
        Tetromino::O(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::I(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::S(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::Z(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::T(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::J(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::L(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4)) => ((x1 -1 , y1 -1), (x2 -1, y2 -1), (x3 -1, y3-1), (x4-1, y4-1))
    };
    if y1 == 0 || y2 == 0  || y3 == 0 || y4 == 0 {
        return true;
    }
    if board.board[(y1 -1) as usize ][x1 as usize] != 0 || board.board[(y2 -1) as usize ][x2 as usize] != 0 || board.board[(y3 -1) as usize ][x3 as usize] != 0 || board.board[(y4 -1) as usize ][x4 as usize] != 0 {
        return true;
    }
    false
}

fn generate_random_tetromino(prev:u8) -> u8{
    let n = fastrand::u8(0..=6);
    if n == prev {
        return fastrand::u8(0..=6);
    }
    n
}

fn render_game (board: &Board, tetromino:&Tetromino, queued:u8, score:u32, level: u32) -> io::Result<()> {
    let mut out = io::stdout();
    
    queue!(out, MoveTo(0, 0))?;
    let mut render = board.board.clone();
     let ((x1, y1), (x2, y2), (x3, y3), (x4, y4)) = match tetromino {
        Tetromino::O(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))        | Tetromino::I(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::S(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::Z(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::T(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::J(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::L(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4)) => ((x1 -1 , y1 -1), (x2 -1, y2 -1), (x3 -1, y3-1), (x4-1, y4-1))
    };
    
    
    render[y1 as usize][x1 as usize] = 1;
    render[y2 as usize][x2 as usize] = 1;
    render[y3 as usize][x3 as usize] = 1;
    render[y4 as usize][x4 as usize] = 1;

    for row in render.iter().rev() {
        queue!(out, Print("\u{2588}"))?;
        for cell in row {
            if *cell != 0 {
                queue!(out,Print("\u{2588}\u{2588}"))?;
            } else {
                queue!(out, Print(". "))?;
            }

        }
        queue!(out, Print("\u{2588}\r\n"))?;
    }
    queue!(out, Print("\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\r\n")).unwrap();
    queue!(out, Print(format!("Score: {}\r\nLevel: {}", score, level)))?;
    queue!(out, MoveTo(24, 1), Print(format!("Level: {}", level)))?;
    queue!(out, MoveTo(24, 2), Print(format!("Score: {}", score)))?;
    queue!(out, MoveTo(24, 3), Print("Next piece:"))?;
    match u8_to_tetromino(queued) {
        Tetromino::O(_, _, _, _, _) => {
            queue!(out, MoveTo(24, 4), Print("\u{2588}\u{2588}\u{2588}\u{2588}  "))?;
            queue!(out, MoveTo(24, 5), Print("\u{2588}\u{2588}\u{2588}\u{2588}  "))?;
            queue!(out, MoveTo(24, 6), Print("      "))?;
            queue!(out, MoveTo(24, 7), Print("      "))?;
        },
        Tetromino::I(_, _, _, _, _) => {
            queue!(out, MoveTo(24, 4), Print("\u{2588}\u{2588}    "))?;
            queue!(out, MoveTo(24, 5), Print("\u{2588}\u{2588}    "))?;
            queue!(out, MoveTo(24, 6), Print("\u{2588}\u{2588}    "))?;
            queue!(out, MoveTo(24, 7), Print("\u{2588}\u{2588}    "))?;
        },
        Tetromino::S(_, _, _, _, _) => {
            queue!(out, MoveTo(24, 4), Print("  \u{2588}\u{2588}\u{2588}\u{2588}"))?;
            queue!(out, MoveTo(24, 5), Print("\u{2588}\u{2588}\u{2588}\u{2588}  "))?;
            queue!(out, MoveTo(24, 6), Print("      "))?;
            queue!(out, MoveTo(24, 7), Print("      "))?;
        },
        Tetromino::Z(_, _, _, _, _) => {
            queue!(out, MoveTo(24, 4), Print("\u{2588}\u{2588}\u{2588}\u{2588}  "))?;
            queue!(out, MoveTo(24, 5), Print("  \u{2588}\u{2588}\u{2588}\u{2588}"))?;
            queue!(out, MoveTo(24, 6), Print("      "))?;
            queue!(out, MoveTo(24, 7), Print("      "))?;
        },
        Tetromino::T(_, _, _, _, _) => {
            queue!(out, MoveTo(24, 4), Print("  \u{2588}\u{2588}  "))?;
            queue!(out, MoveTo(24, 5), Print("\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}"))?;
            queue!(out, MoveTo(24, 6), Print("      "))?;
            queue!(out, MoveTo(24, 7), Print("      "))?;
        },
        Tetromino::J(_, _, _, _, _) => {
            queue!(out, MoveTo(24, 4), Print("  \u{2588}\u{2588}  "))?;
            queue!(out, MoveTo(24, 5), Print("  \u{2588}\u{2588}  "))?;
            queue!(out, MoveTo(24, 6), Print("\u{2588}\u{2588}\u{2588}\u{2588}  "))?;
            queue!(out, MoveTo(24, 7), Print("      "))?;
        },
        Tetromino::L(_, _, _, _, _) => {
            queue!(out, MoveTo(24, 4), Print("\u{2588}\u{2588}    "))?;
            queue!(out, MoveTo(24, 5), Print("\u{2588}\u{2588}    "))?;
            queue!(out, MoveTo(24, 6), Print("\u{2588}\u{2588}\u{2588}\u{2588}  "))?;
            queue!(out, MoveTo(24, 7), Print("      "))?;
        },



    }
    out.flush()

}


fn u8_to_tetromino(val:u8) -> Tetromino {
    match val {
            0 => Tetromino::O(0, (5, 20), (6, 20), (5,19), (6,19)),
            1 => Tetromino::I(0, (5, 20), (5, 19), (5,18), (5,17)),
            2 => Tetromino::S(0, (5, 20), (6, 20), (4,19), (5,19)),
            3 => Tetromino::Z(0, (5, 20), (6, 20), (6,19), (7,19)),
            4 => Tetromino::T(0, (5, 20), (4, 19), (5,19), (6,19)),
            5 => Tetromino::J(0, (5, 20), (5, 19), (4,18), (5,18)),
            6 => Tetromino::L(0, (5, 20), (5, 19), (5,18), (6,18)),
            _ => panic!("tetromino value can only never be higher than 6")
    }

}

fn hard_drop_naive(board: &mut Board, tetromino: &Tetromino) {

    let mut new_val1 = Tetromino::move_down(tetromino);
    let mut new_val2 = new_val1.clone();
    while possible(board, &new_val2){
        new_val1 = new_val2;
        new_val2 = Tetromino::move_down(&new_val2)
    }
    if connected(board, &new_val1) {
        (*board).add_tetromino(&new_val1);
    }
    else {
    panic!("should not be possible");
    }




}

fn hard_drop(board: &mut Board, tetromino: &Tetromino) {
    let ((x1, y1), (x2, y2), (x3, y3), (x4, y4)) = match tetromino {
        Tetromino::O(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::I(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::S(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::Z(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::T(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::J(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4))
        | Tetromino::L(_, (x1, y1), (x2, y2), (x3, y3), (x4, y4)) => ((x1 -1 , y1 -1), (x2 -1, y2 -1), (x3 -1, y3-1), (x4-1, y4-1))
    };
    
    fn calc_highest_y(grid: &[[u8; 10]; 20], x_cor: u8) -> u8 {
        let y_cor = (0..grid.len())
            .rev()
            .find(|&y| grid[y][x_cor as usize ] != 0);

        match y_cor {
            Some(y) => y as u8,
            None => 0,
        }
    }
    let highest_y1 = calc_highest_y(&board.board, x1);
    let highest_y2 = calc_highest_y(&board.board, x2);
    let highest_y3 = calc_highest_y(&board.board, x3);
    let highest_y4 = calc_highest_y(&board.board, x4);
    //println!("{} {} {} {} {} {} {} {}", y1, highest_y1, y2, highest_y2, y3, highest_y3, y4, highest_y4);
    let delta_y1 = y1 - highest_y1;
    let delta_y2 = y2 - highest_y2;
    let delta_y3 = y3 - highest_y3;
    let delta_y4 = y4 - highest_y4;
    let smallest = std::cmp::min(std::cmp::min(delta_y1, delta_y2), std::cmp::min(delta_y3, delta_y4));
    fn make_tetromino (smallest: u8, (x1, y1):(u8,u8), (x2, y2):(u8,u8), (x3, y3):(u8,u8), (x4, y4):(u8,u8)) -> ((u8,u8), (u8,u8), (u8,u8), (u8,u8)) {
        ((x1, y1 - smallest), (x2, y2 - smallest), (x3, y3 - smallest), (x4, y4 - smallest))
    }

    let new_val = match *tetromino {
        Tetromino::O(rotation, c1,c2,c3,c4) => {let (cn1, cn2, cn3, cn4) = make_tetromino(smallest, c1,c2,c3,c4);
                                                Tetromino::O(rotation, cn1, cn2, cn3, cn4)},
        Tetromino::I(rotation, c1,c2,c3,c4) => {let (cn1, cn2, cn3, cn4) = make_tetromino(smallest, c1,c2,c3,c4);
                                                Tetromino::I(rotation, cn1, cn2, cn3, cn4)},
        Tetromino::S(rotation, c1,c2,c3,c4) => {let (cn1, cn2, cn3, cn4) = make_tetromino(smallest, c1,c2,c3,c4);
                                                Tetromino::S(rotation, cn1, cn2, cn3, cn4)},
        Tetromino::Z(rotation, c1,c2,c3,c4) => {let (cn1, cn2, cn3, cn4) = make_tetromino(smallest, c1,c2,c3,c4);
                                                Tetromino::Z(rotation, cn1, cn2, cn3, cn4)},
        Tetromino::T(rotation, c1,c2,c3,c4) => {let (cn1, cn2, cn3, cn4) = make_tetromino(smallest, c1,c2,c3,c4);
                                                Tetromino::T(rotation, cn1, cn2, cn3, cn4)},
        Tetromino::J(rotation, c1,c2,c3,c4) => {let (cn1, cn2, cn3, cn4) = make_tetromino(smallest, c1,c2,c3,c4);
                                                Tetromino::J(rotation, cn1, cn2, cn3, cn4)},
        Tetromino::L(rotation, c1,c2,c3,c4) => {let (cn1, cn2, cn3, cn4) = make_tetromino(smallest, c1,c2,c3,c4);
                                                Tetromino::L(rotation, cn1, cn2, cn3, cn4)},
 
    };


    if connected(board, &new_val) {
        (*board).add_tetromino(&new_val);
        //println!("{}", board)
    } else {
        panic!("should not happen in think")
    }
}
