use std::fmt::Display;
use std::iter::Iterator;

use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{ Style, Color};
use tui::widgets::{Block, Widget};

/// Holds data to be displayed in a Table widget
pub enum Row<D, I>
where
    D: Iterator<Item = I>,
    I: Display,
{
    Data(D),
    StyledData(D, Style),
}


pub struct Table<'a, T, H, I, D, R>
where
    T: Display,
    H: Iterator<Item = T>,
    I: Display,
    D: Iterator<Item = I>,
    R: Iterator<Item = Row<D, I>>,
{
    /// A block to wrap the widget in
    block: Option<Block<'a>>,
    /// Base style for the widget
    style: Style,
    /// Header row for all columns
    header: H,
    /// Style for the header
    header_style: Style,
    /// Width of each column (if the total width is greater than the widget width some columns may
    /// not be displayed)
    widths: &'a [u16],
    height: usize,
    selected: Option<usize>,
    /// Space between each column
    column_spacing: u16,
    /// Data to display in each row
    rows: R,
}

impl<'a, T, H, I, D, R> Default for Table<'a, T, H, I, D, R>
where
    T: Display,
    H: Iterator<Item = T> + Default,
    I: Display,
    D: Iterator<Item = I>,
    R: Iterator<Item = Row<D, I>> + Default,
{
    fn default() -> Table<'a, T, H, I, D, R> {
        Table {
            block: None,
            style: Style::default(),
            header: H::default(),
            header_style: Style::default(),
            widths: &[],
            height: 0,
            selected: None,
            rows: R::default(),
            column_spacing: 1,
        }
    }
   
}

impl<'a, T, H, I, D, R> Table<'a, T, H, I, D, R>
where
    T: Display,
    H: Iterator<Item = T>,
    I: Display,
    D: Iterator<Item = I>,
    R: Iterator<Item = Row<D, I>>,
{
    pub fn new(header: H, rows: R) -> Table<'a, T, H, I, D, R> {
        Table {
            block: None,
            style: Style::default(),
            header,
            header_style: Style::default(),
            widths: &[],
            selected: None,
            height: 0,
            rows,
            column_spacing: 1,
        }
    }
    pub fn block(mut self, block: Block<'a>) -> Table<'a, T, H, I, D, R> {
        self.block = Some(block);
        self
    }

    pub fn header<II>(mut self, header: II) -> Table<'a, T, H, I, D, R>
    where
        II: IntoIterator<Item = T, IntoIter = H>,
    {
        self.header = header.into_iter();
        self
    }

    pub fn header_style(mut self, style: Style) -> Table<'a, T, H, I, D, R> {
        self.header_style = style;
        self
    }

    pub fn widths(mut self, widths: &'a [u16]) -> Table<'a, T, H, I, D, R> {
        self.widths = widths;
        self
    }

    pub fn rows<II>(mut self, rows: II) -> Table<'a, T, H, I, D, R>
    where
        II: IntoIterator<Item = Row<D, I>, IntoIter = R>,
    {
        self.rows = rows.into_iter();
        self
    }

    pub fn style(mut self, style: Style) -> Table<'a, T, H, I, D, R> {
        self.style = style;
        self
    }

    pub fn column_spacing(mut self, spacing: u16) -> Table<'a, T, H, I, D, R> {
        self.column_spacing = spacing;
        self
    }

     pub fn select(mut self, index: Option<usize>) -> Table<'a, T, H,I,D,R> {
        self.selected = index;
        self
    }

}

impl<'a, T, H, I, D, R> Widget for Table<'a, T, H, I, D, R>
where
    T: Display,
    H: Iterator<Item = T>,
    I: Display,
    D: Iterator<Item = I>,
    R: Iterator<Item = Row<D, I>>,
{

   
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        // Render block if necessary and get the drawing area
        let table_area = match self.block {
            Some(ref mut b) => {
                b.draw(area, buf);
                b.inner(area)
            }
            None => area,
        };

        let table_height = table_area.height as usize - 2;

        // Use highlight_style only if something is selected
        let selected = match self.selected {
            Some(i) => Some(i),
            None => None,
        };

        let offset = if let Some(s) = selected {
            if s >= table_height {
                s - table_height + 1
            } else {
                0
            }
        } else {
            0
        };

        // Set the background
        self.background(table_area, buf, self.style.bg);

        // Save widths of the columns that will fit in the given area
        let mut x = 0;
        let mut widths = Vec::with_capacity(self.widths.len());
        for width in self.widths.iter() {
            if x + width < table_area.width {
                widths.push(*width);
            }
            x += *width;
        }

        let mut y = table_area.top();

        // Draw header
        if y < table_area.bottom() {
            x = table_area.left();
            for (w, t) in widths.iter().zip(self.header.by_ref()) {
                buf.set_string(x, y, format!("{}", t), self.header_style);
                x += *w + self.column_spacing;
            }
        }
        y += 2;

        // Draw rows
        let default_style = Style::default();
        if y < table_area.bottom() {
            let remaining = (table_area.bottom() - y) as usize;
            for (i, row) in self.rows.by_ref().skip(offset as usize).take(remaining).enumerate() {
                let (data, style) = match row {
                    Row::Data(d) => (d, default_style),
                    Row::StyledData(d, s) => (d, s),
                };
                x = table_area.left();
                for (w, elt) in widths.iter().zip(data){
                    if let Some(sl) =  self.selected {
                        if sl == i + offset {
                            buf.set_stringn(x, y + i as u16, format!(" > {}", elt), *w as usize, Style::default().fg(Color::Green));
                        } else {
                            buf.set_stringn(x, y + i as u16, format!("   {}", elt), *w as usize, style);
                        }
                    };
                    x += *w + self.column_spacing;
                }

            }
        }
    }
}
