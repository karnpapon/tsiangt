#[allow(dead_code)]
use crate::App::App;
use crate::custom_widgets::{Table as PlaylistTable, Row as PlaylistRow};

use std::io;
use termion::raw::IntoRawMode;
use tui::{ Terminal, Frame };
use tui::backend::{ TermionBackend, Backend };
use tui::widgets::{Widget, Block, Borders, List, Tabs, Row, Table, SelectableList};
use tui::layout::{Layout, Constraint, Direction, Rect};
use tui::style::{Color, Modifier, Style};


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
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());
        Tabs::default()
            .block(Block::default().borders(Borders::ALL).title(app.title))
            .titles(&app.tabs.titles)
            .style(Style::default())
            .highlight_style(Style::default().fg(Color::Yellow))
            .select(app.tabs.index)
            .render(&mut f, chunks[0]);
        match app.tabs.index {
            0 => draw_first_tab(&mut f, &app, chunks[1]),
            1 => draw_second_tab(&mut f, &app, chunks[1]),
            2 => draw_third_tab(&mut f, &app, chunks[1]),
            _ => {}
        };
    })
  }


fn draw_first_tab<B>(f: &mut Frame<B>, app: &App, area: Rect)
    where B: Backend {

    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)] .as_ref())
        .split(area);

    let items = app
        .playlist
        .items
        .iter()
        .map(|item| TableItem {
            id: item.title.to_string(),
            format: vec![
                item.title.to_string().to_owned(),
                item.artist.to_string().to_owned(),
                item.album.to_string().to_owned(),
                //item.duration.to_string().to_owned(),
            ],
        })
        .collect::<Vec<TableItem>>();

    let highlight_state = true;


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
       // TableHeader {
       //     text: "Length",
       //     width: get_percentage_width(area.width, 0.1),
       // },
    ];


    draw_table(
        f,
        app,
        chunks[0],
        ("Playlist", &header),
        &items,
        || app.get_playing_track_index(),
        highlight_state,
    );
}

fn draw_second_tab<B>(f: &mut Frame<B>, app: &App, area: Rect)
        where B: Backend
{
      let chunks = Layout::default()
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .direction(Direction::Horizontal)
        .split(area);
  
    draw_directory(f, app, chunks[0]);
    draw_directory_files(f, app, chunks[1]);
}

fn draw_third_tab<B>(f: &mut Frame<B>, app: &App, area: Rect) 
    where B: Backend
{

    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)] .as_ref())
        .split(area);

     let mut items = Vec::new();

    items.push(TableItem {
        id: String::from( "placeholder" ),
        format: vec!["No Results found..".to_string()]
    });

    let highlight_state = true;


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


    draw_table(
        f,
        app,
        chunks[0],
        ( &app.tabs.panels.titles[1] , &header),
        &items,
        || app.get_playing_track_index(),
        highlight_state,
    );
}



fn draw_directory<B>(f: &mut Frame<B>, app: &App, area: Rect)
    where B: Backend 
{
    let current_title = &app.tabs.panels.titles[0];
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
                .border_style( draw_active(app, current_title))
                .title_style(active)
                .title(app.tabs.panels.titles[app.tabs.index % app.tabs.titles.len() - 1])
            )
            .items(&d)
            .select(Some(app.directory.selected))
            .highlight_style(Style::default().fg(Color::Green))
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


    draw_table(
        f,
        app,
        area,
        (&app.tabs.panels.titles[1], &header),
        &items,
        || app.get_playing_track_index(),
        *&app.tabs.panels.index == 1,
    );
}


// `percentage` param needs to be between 0 and 1
fn get_percentage_width(width: u16, percentage: f32) -> u16 {
    let padding = 3;
    let width = width - padding;
    (f32::from(width) * percentage) as u16
}


fn draw_table<B,F>(
    f: &mut Frame<B>,
    app: &App,
    area: Rect,
    table_layout: (&str, &[TableHeader]), // (title, header colums)
    items: &[TableItem], // The nested vector must have the same length as the `header_columns`
    select_fn: F,
    highlight_state: bool,
) where
    B: Backend,
    F: Fn() -> Option<usize>
{
    let selected_style = get_color(highlight_state);
        //.modifier(Modifier::BOLD);

   

    let rows = items.iter().enumerate().map(|(i, item)| {
        let mut formatted_row = item.format.clone();
        let mut style = Style::default().fg(Color::White); // default styling
     
        // TODO: highlight from widget instead?
        match app.playing_track_index {
            Some(x) => if i == x { style = Style::default().fg(Color::Red);},
            None => {}
        }
        PlaylistRow::StyledData(formatted_row.into_iter(), style)
    });

    
    let (title, header_columns) = table_layout;

    let widths = header_columns.iter().map(|h| h.width).collect::<Vec<u16>>();


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
        .select( select_fn() )
        .render(f, area);
    }


fn get_color(is_active : bool) -> Style {
    match is_active {
        true => Style::default().fg(Color::Green),
        _ => Style::default().fg(Color::White),
    }
}

fn draw_active(app: &App, title: &str) -> Style {

    let active_style = Style::default().fg(Color::Green);
    let inactive_style = Style::default().fg(Color::White);
    let active_panel = if app.tabs.panels.get_title() == title{
         active_style
    } else { 
        inactive_style
    };

    active_panel
}
