
# kanban-rs


## What is 'Kanban'?

Kanban is originally a method of [manufacturing scheduling](https://en.wikipedia.org/wiki/Kanban) but in software development Kanban is a method of [lean task organization](https://en.wikipedia.org/wiki/Kanban_(development)) where tasks are arranged in groups on a board so each problem can be broken down to various entries and solved one at a time.

## Usage


> q = Quit

Quit the board. Will quit even if there are unsaved changes.

> s = Save

Saves the board in the current path where the kanban was first opened. Subsequent saves overwrite the state of the current board.

> c = Create Task

Prompt the creation of a new task in the block the pointer is in.

> d = Delete Task

Delete the task where the pointer is placed on top. No confirmation asked.

> \> = Shift Task to right

Shift the task where the pointer is to the right block.

> < = Shift Task to left

Shift the task where the pointer is to the left block.

> Return = Show Task/Hide Task

Show or Hide task where the pointer is on top of.

## Demo

![](https://github.com/rapha-au/kanban-rs/blob/main/assets/KanbanExample.gif)

