#[allow(dead_code)]
use crate::App::App;
use crate::custom_widgets::{Table as PlaylistTable, Row as PlaylistRow};

use std::io;
use tui::{ Terminal, Frame };
use tui::backend::{ Backend };
use tui::widgets::{Widget, Block, Borders, Tabs, Text, Paragraph, SelectableList};
use tui::layout::{Layout, Constraint, Direction, Alignment, Rect};
use tui::style::{Color,  Style};


pub struct TableHeader<'a> {
    text: &'a str,
    width: u16,
}

#[derive(Debug,Clone)]
pub struct TableItem {
   pub id: String,
   pub format: Vec<String>,
}


pub fn draw<B>(terminal: &mut Terminal<B>, app: &App)  -> Result<(), io::Error>
    where B: Backend {

    terminal.draw(|mut f| {

        let chunks_main = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size()); 

        let chunk_tab = Layout::default()
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .direction(Direction::Horizontal)
            .split(chunks_main[0]);

        let chunk_body = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(chunks_main[1]);

        Tabs::default()
            .block(Block::default().borders(Borders::ALL).title(app.title))
            .titles(&app.tabs.titles)
            .style(Style::default())
            .highlight_style(Style::default().fg(Color::Green))
            .select(app.tabs.index)
            .render(&mut f, chunk_tab[0]);
        draw_search_input(&mut f, &app, chunk_tab[1]);


        match app.tabs.index {
            0 => draw_playlist(&mut f, &app, chunk_body[0]),
            1 => draw_library(&mut f, &app, chunk_body[0]),
            2 => draw_search(&mut f, &app, chunk_body[0]),
            _ => {}
        };
    })
  }


fn draw_playlist<B>(f: &mut Frame<B>, app: &App, area: Rect)
    where B: Backend {

    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)] .as_ref())
        .split(area);


    let is_track_highlighted: bool;

    let mut items = Vec::new();
    if app.playlist.items.len() > 0 {

       is_track_highlighted = true;

       items = app
             .playlist
             .items
             .iter()
             .map(|item| TableItem {
                 id: item.title.to_string(),
                 format: vec![
                     item.title.to_string().to_owned(),
                     item.artist.to_string().to_owned(),
                     item.album.to_string().to_owned(),
                 ],
             })
             .collect::<Vec<TableItem>>();
    } else {
        is_track_highlighted = false;
         items.push(get_init_selection_table_state("No song added.."));
    }

    let highlight_state = false;

    let header = get_header(&area);

    draw_table(
        f,
        app,
        chunks[0],
        ("Playlist", &header),
        &items,
        app.should_select,
        highlight_state,
        is_track_highlighted
    );
}

fn draw_library<B>(f: &mut Frame<B>, app: &App, area: Rect)
        where B: Backend
{
      let chunks = Layout::default()
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .direction(Direction::Horizontal)
        .split(area);
  
    draw_directory(f, app, chunks[0]);
    draw_directory_files(f, app, chunks[1]);
}

fn draw_search<B>(f: &mut Frame<B>, app: &App, area: Rect) 
    where B: Backend
{

    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)] .as_ref())
        .split(area);

     let mut items = Vec::new();

    items.push(get_init_selection_table_state("No Result found.."));

    let highlight_state = false;

    let header = get_header(&area);
   
    draw_table(
        f,
        app,
        chunks[0],
        ( &app.tabs.panels.titles[1] , &header),
        &items,
        false,
        highlight_state,
        false
    );
}



fn draw_search_input<B>(f: &mut Frame<B>, app: &App, area: Rect) 
    where B: Backend
{

     let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)] .as_ref())
        .split(area);


    // Input box
    Paragraph::new([Text::raw(&app.search_input)].iter())
        .style(Style::default().fg(Color::Yellow))
        .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_style(get_color(app.is_search_active))
        )    
        .render(f, chunks[0]);

    //io::stdout().flush().ok();

}


fn draw_directory<B>(f: &mut Frame<B>, app: &App, area: Rect)
    where B: Backend 
{
    let active = get_color(*&app.tabs.panels.index == 0);

    let mut d = Vec::new();

    for directory in &app.directory.items {
        d.push(format!("/{}", directory
            .file_name()
            .unwrap()
            .to_owned()
            .to_os_string()
            .into_string()
            .unwrap()
            )
        )
    }
       
   Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref());
        SelectableList::default()
            .block(
                Block::default()
                .borders(Borders::ALL)
                .border_style(active)
                .title_style(active)
                .title(app.tabs.panels.titles[app.tabs.index % app.tabs.titles.len() - 1])
            )
            .items(&d)
            .select(Some(app.directory.selected))
            .highlight_style(Style::default().fg(Color::Gray))
            .highlight_symbol(">")
            .render(f, area);
}


fn draw_directory_files<B>(f: &mut Frame<B>, app: &App, area: Rect)
    where B: Backend 
{

   let items = app
       .directory_files
       .items
       .iter()
       .map(|item| TableItem {
           id: item.title.to_string(),
           format: vec![
               item.title.to_string().to_owned(),
               item.artist.to_string().to_owned(),
               item.album.to_string().to_owned(),
           ],
       })
       .collect::<Vec<TableItem>>();

    let header = get_header(&area);

    draw_table(
        f,
        app,
        area,
        (&app.tabs.panels.titles[1], &header),
        &items,
        true,
        *&app.tabs.panels.index == 1,
        false
    );
}


fn draw_table<B>(
    f: &mut Frame<B>,
    app: &App,
    area: Rect,
    table_layout: (&str, &[TableHeader]), 
    items: &[TableItem],     
    should_select: bool,
    highlight_state: bool,
    should_active: bool
) where
    B: Backend,
{
    let rows = items.iter().enumerate().map(|(i, item)| {
        let formatted_row = item.format.clone();
        let mut style = Style::default().fg(Color::White); // default styling
     
        // TODO: highlight from widget instead?
        if should_active {
            match app.playing_track_index {
                Some(x) => if i == x { style = Style::default().fg(Color::Green);},
                None => {}
            }
        }
        
        PlaylistRow::StyledData(formatted_row.into_iter(), style)
    });

    
    let (title, header_columns) = table_layout;
    let select_active: Style;

    let widths = header_columns.iter().map(|h| h.width).collect::<Vec<u16>>();

    let symbol = get_symbol(app.is_playlist_added);
    select_active = get_select_active_color(app.is_playlist_added);


    let select: Option<usize>;
    if should_select {
        select = app.get_playing_track_index();
    } else {
       select = None;
    };

    PlaylistTable::new(header_columns.iter().map(|h| h.text), rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title(title)
                .title_style(get_color(highlight_state))
                .border_style(get_color(highlight_state)),
        )
        .style(Style::default().fg(Color::White))
        .widths(&widths)
        .select( select )
        .set_select_active_style(select_active)
        .select_symbol(symbol)
        .render(f, area);
    }


//--------------------------------------
//------------- helpers ----------------.
//--------------------------------------
//


// `percentage` param needs to be between 0 and 1
fn get_percentage_width(width: u16, percentage: f32) -> u16 {
    let padding = 3;
    let width = width - padding;
    (f32::from(width) * percentage) as u16
}


fn get_color(is_active : bool) -> Style {
    match is_active {
        true => Style::default().fg(Color::Green),
        _ => Style::default().fg(Color::White),
    }
}

fn get_symbol(state: bool) -> String {
    match state {
        true => { "+".to_string()},
        _ => { ">".to_string()}
    }
}


fn get_select_active_color(state: bool) -> Style {
    match state {
        true => {Style::default().fg(Color::Green)},
        _ => {Style::default().fg(Color::Gray)}
    }
        
}

fn get_header(area: &Rect) -> [TableHeader; 3]{

  let header = [
        TableHeader {
            text: "  Title",
            width: get_percentage_width(area.width, 0.33),
        },
        TableHeader {
            text: "Artist",
            width: get_percentage_width(area.width, 0.33),
        },
        TableHeader {
            text: "Album",
            width: get_percentage_width(area.width, 0.33),
        },
];

  header

}

fn get_init_selection_table_state(placeholder: &str) -> TableItem{
  let item = TableItem {
        id: String::from( "placeholder" ),
        format: vec![placeholder.to_string()]
    };

  item
}
