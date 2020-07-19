//! Simple unicode table in-terminal rendering for debug and linter info

#![allow(unused_must_use)]

use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// A structure representing a single cell inside of a table, this could be a heading, or a row element
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell<'a> {
    pub text: &'a str,
    pub color: Option<Color>,
}

impl<'a> From<&'a str> for Cell<'a> {
    fn from(string: &'a str) -> Self {
        Cell {
            text: string,
            color: None,
        }
    }
}

impl<'a> Cell<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            color: None,
        }
    }

    pub fn with_color(text: &'a str, color: Color) -> Self {
        Self {
            text,
            color: Some(color),
        }
    }
}

/// A simple but limited unicode table renderer for rendering tables in the terminal
/// The first cell of a row is used as the heading, any extra row elements will be thrown out,
/// any missing row elements will be rendered as empty
pub struct Table<'a> {
    pub columns: Vec<Cell<'a>>,
    pub rows: Vec<Vec<Cell<'a>>>,
    pub notes: Vec<Cell<'a>>,
    renderer: StandardStream,
}

impl<'a> Table<'a> {
    pub fn new(columns: Vec<Cell<'a>>, rows: Vec<Vec<Cell<'a>>>, notes: Vec<Cell<'a>>) -> Self {
        Self {
            columns,
            rows,
            notes,
            renderer: StandardStream::stdout(ColorChoice::Always),
        }
    }

    /// Get the total width of the table (in single characters) once fully rendered
    pub fn get_total_row_len(&self) -> usize {
        self.get_column_sizes().iter().sum::<usize>() + self.columns.len() + 1
    }

    /// Get the max size of each column's inside, this includes 2 spaces and the max column text size
    pub fn get_column_sizes(&self) -> Vec<usize> {
        let mut res = Vec::with_capacity(self.columns.len());

        for (idx, column) in self.columns.iter().enumerate() {
            let mut max_len = column.text.len();

            for row in self.rows.iter() {
                if row.get(idx).map(|cell| cell.text.len()).unwrap_or(0) > max_len {
                    max_len = row[idx].text.len();
                }
            }
            if idx == self.columns.len() {
                // Right pipe if its the last cell in the row
                max_len += 1;
            }

            res.push(max_len + 2);
        }
        res
    }

    /// Render the top border of the table
    pub fn render_top(&mut self) {
        write!(&mut self.renderer, "╒")
        .expect("Failed to write to stdout");

        let column_sizes = self.get_column_sizes();

        for (idx, cell_len) in column_sizes.iter().enumerate() {
            write!(&mut self.renderer, "{}", "═".repeat(*cell_len))
                .expect("Failed to write to stdout");

            if idx != column_sizes.len() - 1 {
                write!(&mut self.renderer, "╤")
                    .expect("Failed to write to stdout");
            } else {
                writeln!(&mut self.renderer, "╕")
                    .expect("Failed to write to stdout");
            }
        }
    }

    /// Render the bottom border of the table
    pub fn render_bottom(&mut self, notes: bool) {
        if notes {
            write!(&mut self.renderer, "├")
                .expect("Failed to write to stdout");
        } else {
            write!(&mut self.renderer, "└")
                .expect("Failed to write to stdout");
        }

        let column_sizes = self.get_column_sizes();
    
        for (idx, cell_len) in column_sizes.iter().enumerate() {
            write!(&mut self.renderer, "{}", "─".repeat(*cell_len))
                .expect("Failed to write to stdout");

            if idx != column_sizes.len() - 1 {
                write!(&mut self.renderer, "┴")
                    .expect("Failed to write to stdout");
            } else {
                writeln!(&mut self.renderer, "┘")
                    .expect("Failed to write to stdout");
            }
        }
    }

    /// Render the middle of a table, including the left and right borders
    pub fn render_middle(&mut self, double_lines: bool) {
        let (left, mid, right, intersect) = if double_lines {
            ("╞", "═", "╡", "╪")
        } else {
            ("├", "─", "┤", "┼")
        };

        write!(&mut self.renderer, "{}", left)
            .expect("Failed to write to stdout");

        let column_sizes = self.get_column_sizes();
        
        for (idx, cell_len) in column_sizes.iter().enumerate() {
            write!(&mut self.renderer, "{}", mid.repeat(*cell_len))
                .expect("Failed to write to stdout");

            if idx != column_sizes.len() - 1 {
                write!(&mut self.renderer, "{}", intersect)
                    .expect("Failed to write to stdout");
            } else {
                writeln!(&mut self.renderer, "{}", right)
                    .expect("Failed to write to stdout");
            }
        }
    }

    /// Render a row of cells in the table
    pub fn render_row_cells(&mut self, rows: Vec<Cell>) {
        let column_sizes = self.get_column_sizes();

        for (idx, column) in column_sizes.iter().enumerate() {
            write!(&mut self.renderer, "│ ")
                .expect("Failed to write to stdout");

            if let Some(cell) = rows.get(idx) {
                if let Some(color) = cell.color {
                    self.renderer.set_color(ColorSpec::new().set_fg(Some(color)));

                    write!(&mut self.renderer, "{}", cell.text)
                        .expect("Failed to write to stdout");

                    self.renderer.set_color(&ColorSpec::new());
                } else {
                    write!(&mut self.renderer, "{}", cell.text)
                        .expect("Failed to write to stdout");
                }
                
                let trailing_spaces = (column - 2) - cell.text.len();

                write!(&mut self.renderer, " {}", " ".repeat(trailing_spaces))
                    .expect("Failed to write to stdout");
            } else {
                write!(&mut self.renderer, "{}", " ".repeat(column - 1))
                    .expect("Failed to write to stdout");
            }
        }

        writeln!(&mut self.renderer, "│")
            .expect("Failed to write to stdout");
    }

    pub fn render_notes(&mut self) {
        for note in self.notes.iter() {
            self.renderer.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)));

            write!(&mut self.renderer, "= ")
                .expect("Failed to write to stdout");
            
            self.renderer.set_color(&ColorSpec::new());

            if let Some(color) = note.color {
                self.renderer.set_color(ColorSpec::new().set_fg(Some(color)));

                writeln!(&mut self.renderer, "{}", note.text)
                    .expect("Failed to write to stdout");

                self.renderer.set_color(&ColorSpec::new());
            } else {
                writeln!(&mut self.renderer, "{}", note.text)
                    .expect("Failed to write to stdout");
            }
        }
        
    }

    /// Render the table to the terminal
    pub fn render(&mut self) {
        self.render_top();
        
        self.render_row_cells(self.columns.to_owned());
        
        if self.rows.is_empty() {
            if self.notes.is_empty() {
                return self.render_bottom(false);
            } else {
                self.render_bottom(true);
                return self.render_notes();
            }
        }

        self.render_middle(true);
        self.render_row_cells(self.rows[0].to_owned());
        
        for row in self.rows.to_owned().into_iter().skip(1) {
            self.render_middle(false);
            self.render_row_cells(row);
        }

        if self.notes.is_empty() {
            self.render_bottom(false);
        } else {
            self.render_bottom(true);
            self.render_notes();
        }

        writeln!(&mut self.renderer, "");
    }
}
