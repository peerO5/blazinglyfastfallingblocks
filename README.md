# (Blazingly Fast) Tetris

## user manual

use `q` and `e` to rotate clockwise and counter - clockwise
use ` ` to hard drop a tetromino
use the down arrow to soft drop a tetromino
use the left and right arrow to move the tetromino to the right and left


## level calculation
at the beginning you start at level 0 per 10 rows cleared your level increases by 1

## score calculation

the score is calculated based on the rows cleared mutliplied with the level

| Rows cleared | Score calulation |
|--------------| -----------------|
| 1 | 40 * (level + 1)   |
| 2 | 100 * (level + 1)  |
| 3 | 300 * (level + 1)  |
| 4 | 1200 * (level + 1) |

## passive drop speed

a tetromino passive drop speed is dependend on the level the drop speed is the amount of ms it takes for the tetromino to drop 1 row

| level | drop speed |
|-------|------------|
| 0 | 798 |
| 1 | 715 |
| 2 | 632 |
| 3 | 549 |
| 4 | 466 |
| 5 | 383 |
| 6 | 300 |
| 7 | 216 |
| 8 | 133 |
| 9 | 100 |
| 10-12 | 83 |
| 13-15 | 67 |
| 16-18 | 50 |
| 19-28 | 33 |
| 29+ | 17 |

the board is redrawn at ~60 fps ( 16 ms)


