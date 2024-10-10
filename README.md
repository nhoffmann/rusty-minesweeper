# Minesweeper

A Minesweeper clone written in Rust and Bevy. Mainly for learning Rust and Bevy.

## Minimum Viable Products

* [x] The window is filled with a grid of tiles
* [x] There is a set number of difficulties. Mines are scattered randomly over the board
    * Beginner: 9x9, 10 mines
    * Intermediate: 16x16, 40 mines
    * Expert: 30x16, 99 mines 
* [x] The player left clicks tiles and reveals their content
    * [x] When the tile is not adjacent to a mine tile (empty tile), it reveals all adjacent empty tiles consecutively
    * [x] When the tile is adjacent to a mine tile, it displays the number of mine tiles it is adjacent to
    * [x] When the tile is a mine tile, the game is lost
* [x] The player right clicks tiles to toggle flagging them as potential mines, a marked tile can still be opened
* [x] When all non mine tiles have been revealed, the game ends victoriously
