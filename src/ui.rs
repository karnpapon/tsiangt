#[allow(dead_code)]
use crate::App::App;

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


pub struct TableItem {
    id: String,
    format: Vec<String>,
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
                item.duration.to_string().to_owned(),
            ],
        })
        .collect::<Vec<TableItem>>();

    let highlight_state = true;


    let header = [
        TableHeader {
            text: "  Title",
            width: get_percentage_width(area.width, 0.3),
        },
        TableHeader {
            text: "Artist",
            width: get_percentage_width(area.width, 0.3),
        },
        TableHeader {
            text: "Album",
            width: get_percentage_width(area.width, 0.25),
        },
        TableHeader {
            text: "Length",
            width: get_percentage_width(area.width, 0.1),
        },
    ];


    draw_table(
        f,
        app,
        chunks[0],
        ("Playlist", &header),
        &items,
        app.playlist.selected,
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
  
    draw_pools(f, app, chunks[1]);
    draw_lists(f, app, chunks[0]);
}

fn draw_pools<B>(f: &mut Frame<B>, app: &App, area: Rect)
    where B: Backend 
{
    SelectableList::default()
        .block(
            Block::default()
            .borders(Borders::ALL)
            .border_style(draw_active(app, "Files"))
            .title(app.tabs.panels.titles[app.tabs.index % app.tabs.titles.len()])
        )
        .items(&app.pools.items)
        .select(Some(app.pools.selected))
        .highlight_style(Style::default().fg(Color::Green))
        .highlight_symbol(">")
        .render(f, area);
}


fn draw_lists<B>(f: &mut Frame<B>, app: &App, area: Rect)
    where B: Backend 
{
       
   Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref());
        SelectableList::default()
            .block(
                Block::default()
                .borders(Borders::ALL)
                .border_style( draw_active(app, "Directory"))
                .title(app.tabs.panels.titles[app.tabs.index % app.tabs.titles.len() - 1])
            )
            .items(&app.directory.items )
            .select(Some(app.directory.selected))
            .highlight_style(Style::default().fg(Color::Green))
            .highlight_symbol(">")
            .render(f, area);
}


// `percentage` param needs to be between 0 and 1
fn get_percentage_width(width: u16, percentage: f32) -> u16 {
    let padding = 3;
    let width = width - padding;
    (f32::from(width) * percentage) as u16
}


fn draw_table<B>(
    f: &mut Frame<B>,
    app: &App,
    area: Rect,
    table_layout: (&str, &[TableHeader]), // (title, header colums)
    items: &[TableItem], // The nested vector must have the same length as the `header_columns`
    selected_index: usize,
    highlight_state: bool,
) where
    B: Backend,
{
    let selected_style = get_color(highlight_state);
        //.modifier(Modifier::BOLD);
   // let mut track_playing_index: bool = false;


   // for (item_index , item) in app.playlist.items.iter().enumerate(){
   //     if item_index == selected_index {
   //        track_playing_index = true
   //     } else {
   //        track_playing_index = false
   //     }
   // };

    let rows = items.iter().enumerate().map(|(i, item)| {
        let mut formatted_row = item.format.clone();
        let mut style = Style::default().fg(Color::White); // default styling
            //.;

        if i == selected_index {
            formatted_row[0] = format!(" > {}", &formatted_row[0]);
            style = selected_style;
        } else {
            formatted_row[0] = format!("   {}", &formatted_row[0]);
        }

      // if item.id == 2 {
      //     style = Style::default().fg(Color::Red);
      // }
         //Return row styled data
        Row::StyledData(formatted_row.into_iter(), style)
    });

    let (title, header_columns) = table_layout;

    let widths = header_columns.iter().map(|h| h.width).collect::<Vec<u16>>();

    Table::new(header_columns.iter().map(|h| h.text), rows)
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
        .render(f, area);
    }


fn get_color(is_active : bool) -> Style {
    match is_active {
        true => Style::default().fg(Color::Green),
        _ => Style::default().fg(Color::Gray),
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