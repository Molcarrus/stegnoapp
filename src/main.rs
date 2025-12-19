mod errors;
mod utils;
mod encoder;
mod decoder;

use std::io::{self, stdout};
use std::path::{Path, PathBuf};
use ratatui_explorer::FileExplorer;
use structopt::StructOpt;

use ratatui::Terminal;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::CrosstermBackend;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use crate::decoder::Decoder;
use crate::encoder::Encoder;
use crate::errors::Error;
use crate::utils::ByteMask;

#[derive(StructOpt)]
enum Command {
    Encode {
        #[structopt(parse(from_os_str))]
        image: PathBuf,
        #[structopt(parse(from_os_str))]
        secret: PathBuf,
        #[structopt(parse(from_os_str))]
        output: PathBuf,
    },
    Decode {
        #[structopt(parse(from_os_str))]
        image: PathBuf,
        #[structopt(parse(from_os_str))]
        output: PathBuf,
    }
}

#[derive(StructOpt)]
#[structopt(
    name = "stegnoapp",
    about = "Picture secret stegnography encoder/decoder"
)]
struct Opt {
    #[structopt(short = "b", long = "bits", default_value = "2")]
    bits: u8,
    #[structopt(subcommand)]
    cmd: Command,
}


#[derive(PartialEq, Clone, Copy, Debug)]
enum Screen {
    MainMenu,
    Encode,
    Decode,
    Settings,
    Help,
    Quit,
    FileExplorer,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Purpose {
    EncodeImage,
    EncodeSecret,
    EncodeOutput,
    DecodeImage,
    DecodeOutput
}

struct App {
    curr_screen: Screen,
    prev_screen: Option<Screen>,
    encode_image_input: Option<PathBuf>,
    encode_secret_input: Option<PathBuf>,
    encode_output_input: Option<PathBuf>,
    encode_bits: u8,
    decode_image_input: Option<PathBuf>,
    decode_output_input: Option<PathBuf>,
    decode_bits: u8,
    status: String,
    menu_index: usize,
    file_explorer: Option<FileExplorer>,
    explorer_purpose: Option<Purpose>,
}

impl Default for App {
    fn default() -> Self {
        App {
            curr_screen: Screen::MainMenu,
            prev_screen: None,
            encode_image_input: None,
            encode_secret_input: None,
            encode_output_input: Some(PathBuf::from("stego.png")),
            encode_bits: 2,
            decode_image_input: None,
            decode_output_input: Some(PathBuf::from("extracted.txt")),
            decode_bits: 2,
            status: "Ready | Use Tab/Arrows to navigate, Enter to select".to_string(),
            menu_index: 0,
            file_explorer: None,
            explorer_purpose: None,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let opt = Opt::from_args();
    
    // let mask = ByteMask::new(opt.bits)?;
    
    // match opt.cmd {
    //     Command::Encode { 
    //         image, 
    //         secret, 
    //         output 
    //     } => encode(image, secret, output, mask)?,
    //     Command::Decode { 
    //         image, 
    //         output 
    //     } => decode(image, output, mask)?,
    // }
    // 
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = App::default();
    let res = run_app(&mut terminal, &mut app);
    
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    
    if let Err(err) = res {
        println!("{:?}", err);
    }
    
    Ok(())
}

fn encode(
    image: PathBuf,
    secret: PathBuf,
    output: PathBuf,
    mask: ByteMask
) -> Result<(), Error> {
    let mut encoder = Encoder::new(image, secret, mask)?;
    encoder.save(output)?;
    Ok(())
}

fn decode(
    image: PathBuf, 
    output: PathBuf, 
    mask: ByteMask
) -> Result<(), Error> {
    let decoder = Decoder::new(image, mask)?;
    decoder.save(output)?;
    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App 
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.curr_screen {
                    Screen::MainMenu => handle_main_menu_events(app, key.code),
                    Screen::Encode => handle_encode_events(app, key.code)?,
                    Screen::Decode => handle_decode_events(app, key.code)?,
                    Screen::FileExplorer => handle_file_explorer_events(app, key.code)?,
                    _ => {}
                }
                if app.curr_screen == Screen::Quit {
                    return Ok(());
                }
                if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                    return Ok(());
                }
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());
    
    let menu_titles = vec!["Encode", "Decode", "Settings", "Help", "Quit"];
    let tabs = Tabs::new(menu_titles.iter().cloned().map(|s| s.to_string()).collect::<Vec<_>>())
        .block(Block::default().title("Stegnoapp").borders(Borders::ALL))
        .select(app.menu_index)
        .highlight_style(Style::default().fg(ratatui::style::Color::Yellow));
    f.render_widget(tabs, chunks[0]);
    
    match app.curr_screen {
        Screen::MainMenu => {
            let welcome = Paragraph::new("Select an option from the menu above.\nPress Enter to confirm")
                .block(Block::default().borders(Borders::ALL).title("Main Menu"));
            f.render_widget(welcome, chunks[1]);
        }
        Screen::Encode => {
            let sub_chunks = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25)])
                .split(chunks[1]);
            
            let image_path_str = app.encode_image_input.as_ref().map(|p| p.display().to_string()).unwrap_or("Not selected (press 'i' to select)".to_string());
            let image_input = Paragraph::new(image_path_str)
                .block(Block::default().title("Cover Image Path").borders(Borders::ALL));
            f.render_widget(image_input, sub_chunks[0]);
            
            let secret_path_str = app.encode_secret_input.as_ref().map(|p| p.display().to_string()).unwrap_or("Not selected (press 's' to select)".to_string());
            let secret_input = Paragraph::new(secret_path_str)
                .block(Block::default().title("Secret File Path").borders(Borders::ALL));
            f.render_widget(secret_input, sub_chunks[1]);
            
            let output_path_str = app.encode_output_input.as_ref().map(|p| p.display().to_string()).unwrap_or("Not selected (press 'o' to select)".to_string());
            let output_input = Paragraph::new(output_path_str)
                .block(Block::default().title("Output Path").borders(Borders::ALL));
            f.render_widget(output_input, sub_chunks[2]);
            
            let bits_display = Paragraph::new(format!("Bits: {}", app.encode_bits))
                .block(Block::default().title("LSB Bits (Up/Down to change)").borders(Borders::ALL));
            f.render_widget(bits_display, sub_chunks[3]);
        }
        Screen::Decode => {
            let sub_chunks = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(33)])
                .split(chunks[1]);
            
            let image_path_str = app.decode_image_input.as_ref().map(|p| p.display().to_string()).unwrap_or("Not selected (press 'i' to select)".to_string());
            let image_input = Paragraph::new(image_path_str)
                .block(Block::default().title("Stego Image Path").borders(Borders::ALL));
            f.render_widget(image_input, sub_chunks[0]);
            
            let output_path_str = app.decode_output_input.as_ref().map(|p| p.display().to_string()).unwrap_or("Not selected (press 'o' to select)".to_string());
            let output_input = Paragraph::new(output_path_str)
                .block(Block::default().title("Output Path").borders(Borders::ALL));
           f.render_widget(output_input, sub_chunks[1]);
          
          let bits_display = Paragraph::new(format!("Bits: {}", app.decode_bits))
              .block(Block::default().title("LSB Bits (Up/Down to Change)").borders(Borders::ALL));
          f.render_widget(bits_display, sub_chunks[2]);
        }
        Screen::FileExplorer => {
            if let Some(explorer) = &app.file_explorer {
                let widget = explorer.widget();
                f.render_widget(&widget, chunks[1]);
            }
        }
        _ => {}
    }
    
    let status_bar = Paragraph::new(app.status.as_str())
        .style(Style::default().bg(ratatui::style::Color::Blue).fg(ratatui::style::Color::White));
    f.render_widget(status_bar, chunks[2]);
}

fn handle_main_menu_events(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Left => app.menu_index = app.menu_index.saturating_sub(1),
        KeyCode::Right => if app.menu_index < 4 { app.menu_index += 1 },
        KeyCode::Enter => {
            app.curr_screen = match app.menu_index {
                0 => Screen::Encode,
                1 => Screen::Decode,
                2 => Screen::Settings,
                3 => Screen::Help,
                4 => Screen::Quit,
                _ => Screen::MainMenu,
            };
            app.status = format!("Entered {}", format!("{:?}", app.curr_screen));
        }
        _ => {},
    }
} 

fn handle_encode_events(app: &mut App, code: KeyCode) -> io::Result<()> {    
    match code {
        KeyCode::Char('i') => {
            app.prev_screen = Some(Screen::Encode);
            app.curr_screen = Screen::FileExplorer;
            app.explorer_purpose = Some(Purpose::EncodeImage);
            app.file_explorer = Some(FileExplorer::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?);
            app.status = "Navigate and press Enter to select file, Backspace to cancel".to_string();
        }
        KeyCode::Char('s') => {
            app.prev_screen = Some(Screen::Encode);
            app.curr_screen = Screen::FileExplorer;
            app.explorer_purpose = Some(Purpose::EncodeSecret);
            app.file_explorer = Some(FileExplorer::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?);
            app.status = "Navigate and press Enter to select file, Backspace to cancel".to_string();
        }
        KeyCode::Char('o') => {
            app.prev_screen = Some(Screen::Encode);
            app.curr_screen = Screen::FileExplorer;
            app.explorer_purpose = Some(Purpose::EncodeOutput);
            app.file_explorer = Some(FileExplorer::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?);
            app.status = "Navugate and press Enter to select file, Backspace to cancel".to_string();
        }
        KeyCode::Up => app.encode_bits = (app.encode_bits % 8) + 1,
        KeyCode::Down => app.encode_bits = if app.encode_bits > 1 { app.encode_bits - 1 } else { 8 },
        KeyCode::Enter => {
            if let (Some(image), Some(secret), Some(output)) = (&app.encode_image_input, &app.encode_secret_input, &app.encode_output_input) {
                let mask = match ByteMask::new(app.encode_bits) {
                    Ok(m) => m,
                    Err(e) => {
                        app.status = format!("Error: {}", e);
                        return Ok(());
                    }
                };
                if let Err(e) = encode(image.clone(), secret.clone(), output.clone(), mask) {
                    app.status = format!("Encode failed: {}", e);
                } else {
                    app.status = "Encode successful!".to_string();
                }
            } else {
                app.status = "Please select all paths first".to_string();
            }
        }
        KeyCode::Backspace => app.curr_screen = Screen::MainMenu,
        _ => {}
    }
    
    Ok(())
}

fn handle_decode_events(app: &mut App, code: KeyCode) -> io::Result<()> {
    match code {
        KeyCode::Char('i') => {
            app.prev_screen = Some(Screen::Decode);
            app.curr_screen = Screen::FileExplorer;
            app.explorer_purpose = Some(Purpose::DecodeImage);
            app.file_explorer = Some(FileExplorer::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?);
            app.status = "Navigate and press Enter to select the file, Backspace to cancel".to_string();
        }
        KeyCode::Char('o') => {
            app.prev_screen = Some(Screen::Decode);
            app.curr_screen = Screen::FileExplorer;
            app.explorer_purpose = Some(Purpose::DecodeOutput);
            app.file_explorer = Some(FileExplorer::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?);
            app.status = "Navigate and press Enter to select location (file or dir), Backspace to cancel".to_string();
        }
        KeyCode::Up => app.decode_bits = (app.decode_bits % 8) + 1,
        KeyCode::Down => app.decode_bits = if app.decode_bits > 1 { app.decode_bits - 1 } else { 8 },
        KeyCode::Enter => {
            if let (Some(image), Some(output)) = (&app.decode_image_input, &app.decode_output_input) {
                let mask = match ByteMask::new(app.decode_bits) {
                    Ok(m) => m,
                    Err(e) => {
                        app.status = format!("Error: {}", e);
                        return Ok(());
                    }
                };
                if let Err(e) = decode(image.clone(), output.clone(), mask) {
                    app.status = format!("Decode failed: {}", e);
                } else {
                    app.status = "Please select all paths first".to_string();
                }
            }
        }
        KeyCode::Backspace => app.curr_screen = Screen::MainMenu,
        _ => {}
    }
    
    Ok(())
}

fn handle_file_explorer_events(app: &mut App, code: KeyCode) -> io::Result<()> {
    if let Some(explorer) = app.file_explorer.as_mut() {
        let evt = Event::Key(event::KeyEvent::from(code));
        if let Err(e) = explorer.handle(&evt) {
            app.status = format!("Error: {}", e);
        }
        
        if code == KeyCode::Enter {
            let selected = explorer.current().path().to_path_buf();
            let is_dir = explorer.current().is_dir();
            if let Some(purpose) = app.explorer_purpose {
                let path = if is_dir {
                    match purpose {
                        Purpose::EncodeOutput => selected.join("stego.png"),
                        Purpose::DecodeOutput => selected.join("extracted.txt"),
                        _ => {
                            app.status = "Please select a file, not a directory".to_string();
                            return Ok(());
                        }
                    }
                } else {
                    match purpose {
                        Purpose::EncodeImage | Purpose::EncodeSecret | Purpose::DecodeImage => selected,
                        Purpose::EncodeOutput | Purpose::DecodeOutput => selected,
                    }
                };
                match purpose {
                    Purpose::EncodeImage => app.encode_image_input = Some(path),
                    Purpose::EncodeSecret => app.encode_secret_input = Some(path),
                    Purpose::EncodeOutput => app.encode_output_input = Some(path),
                    Purpose::DecodeImage => app.decode_image_input = Some(path),
                    Purpose::DecodeOutput => app.decode_output_input = Some(path)
                }
                if let Some(prev) = app.prev_screen  {
                    app.curr_screen = prev;
                }
                app.file_explorer = None;
                app.explorer_purpose = None;
            }
        } else if code == KeyCode::Backspace {
            if let Some(prev) = app.prev_screen {
                app.curr_screen = prev;
            }
            app.file_explorer = None;
            app.explorer_purpose = None;
        }
    }
    
    Ok(())
}
