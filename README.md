# issues

1. hard_drop does not work yet it clips weirdly it probs has something to do with the calucation.
2. rotating I in the left gives an error since you get buffer overflow, since when you rotate it is sometimes -2 if it is at index 0 (which is saved as 1 in the Tetromino enum than 1 - 2 = -1 which is not valid for u8
3. moving blocks allows for going trough blocks sometimes fn possible() might need to be adapted
4. the timed going down cant check connected like it is now since now you dont have time to move it to the right of left when it is touching but not yet placed so it should be if not possible (check iteration later) i implemented a gut feeling fix might not work
