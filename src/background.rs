pub struct BackgroundPanel {
    grid: Vec<BackgroundTile>,
    width: std::num::NonZeroUsize,
    height: std::num::NonZeroUsize,
}

impl BackgroundPanel {
    pub fn new(width: std::num::NonZeroUsize, height: std::num::NonZeroUsize) -> Self {
        let grid = Self::generate_grid(width, height);
        BackgroundPanel {
            grid,
            width: width,
            height: height,
        }
    }

    pub fn regenerate(&mut self) {
        self.grid = Self::generate_grid(self.width, self.height)
    }

    fn generate_grid(width: std::num::NonZeroUsize, height: std::num::NonZeroUsize) -> Vec<BackgroundTile> {
        let wfc_grid = pistis::Grid::<WfcTile>::new(width, height);
        let mut grid = match wfc_grid.collapse() {
            Ok(grid) => grid
                .rows()
                .map(|row| row.iter())
                .flatten()
                .map(|tile| BackgroundTile {
                    symbol: tile.symbol,
                    color: ratatui::style::Color::DarkGray,
                    connections: tile.borders,
                })
                .collect::<Vec<_>>(),
            Err(_) => (0..width.get() * height.get())
                .map(|_| BackgroundTile {
                    symbol: '!',
                    color: ratatui::style::Color::White,
                    connections: [false; 4],
                })
                .collect::<Vec<_>>(),
        };

        /* Find the biggest curves */
        let mut curves = Vec::new();
        let mut tiles_to_curves = std::collections::HashMap::new();

        for (start_index, tile) in grid.iter().enumerate() {
            let start_x = start_index % width.get();
            let start_y = start_index / width.get();

            if tiles_to_curves.contains_key(&(start_x, start_y)) {
                /* Curve already explored */
                continue;
            }

            let curve_index = curves.len();
            let mut curve = std::collections::HashSet::new();
            let mut curve_to_explore = vec![((start_x, start_y), tile)];

            while let Some(((x, y), tile)) = curve_to_explore.pop() {
                let index = x + width.get() * y;
                curve.insert((x, y));
                tiles_to_curves.insert((x, y), curve_index);
                if tile.connections[0] && y > 0 {
                    let neighbor_coord = (x, y - 1);
                    if !curve.contains(&neighbor_coord) {
                        curve_to_explore.push((neighbor_coord, &grid[index - width.get()]));
                    }
                }
                if tile.connections[1] && x < width.get() - 1 {
                    let neighbor_coord = (x + 1, y);
                    if !curve.contains(&neighbor_coord) {
                        curve_to_explore.push((neighbor_coord, &grid[index + 1]));
                    }
                }
                if tile.connections[2] && y < height.get() - 1 {
                    let neighbor_coord = (x, y + 1);
                    if !curve.contains(&neighbor_coord) {
                        curve_to_explore.push((neighbor_coord, &grid[index + width.get()]));
                    }
                }
                if tile.connections[3] && x > 0 {
                    let neighbor_coord = (x - 1, y);
                    if !curve.contains(&neighbor_coord) {
                        curve_to_explore.push((neighbor_coord, &grid[index - 1]));
                    }
                }
            }

            curves.push(curve);
        }

        curves.sort_by(|c1, c2| c2.len().cmp(&c1.len()));

        for (x, y) in curves[0].iter() {
            grid[x + width.get() * y].color = ratatui::style::Color::Gray;
        }

        grid
    }
}

impl ratatui::widgets::Widget for &BackgroundPanel {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        for y in 0..(area.height.min(self.height.get() as u16)) {
            for x in 0..(area.width.min(self.width.get() as u16)) {
                let position = ratatui::layout::Position { x, y };
                let index = usize::from(y) * self.width.get() + usize::from(x);
                match buf.cell_mut(position) {
                    Some(cell) => {
                        cell.set_char(self.grid[index].symbol);
                        cell.set_fg(self.grid[index].color);
                    }
                    None => {}
                }
            }
        }
    }
}

struct BackgroundTile {
    symbol: char,
    color: ratatui::style::Color,
    connections: [bool; 4],
}

struct WfcTile {
    symbol: char,
    /// Top, Right, Bottom, Left
    borders: [bool; 4],
}

impl pistis::Tile for WfcTile {
    type Border = bool;
    fn border(&self, direction: pistis::Direction) -> &Self::Border {
        &self.borders[direction.as_index()]
    }
    fn all() -> impl Iterator<Item = Self> {
        let tiles = [
            WfcTile {
                borders: [false, false, false, false],
                symbol: ' ',
            },
            WfcTile {
                borders: [false, true, false, true],
                symbol: '─',
            },
            WfcTile {
                borders: [false, true, true, false],
                symbol: '╭',
            },
            WfcTile {
                borders: [true, false, false, true],
                symbol: '╯',
            },
            WfcTile {
                borders: [false, false, true, true],
                symbol: '╮',
            },
            WfcTile {
                borders: [true, true, false, false],
                symbol: '╰',
            },
        ];
        tiles.into_iter()
    }
}
