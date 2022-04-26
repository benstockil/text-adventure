# Text Adventure Builder!

This is a small app to build text adventure games. It is currently in alpha state, so expect bugs and missing features.

## Writing stories 

Stories are written in .story files.

Commands occur at the beginning of lines, and are prefixed with `+`.

The current list of commands is:

| Command         | Description                                            |
| --------------- | ------------------------------------------------------ |
| `CLEAR`         | Clears the screen                                      |
| `INPUT: [name]` | Receives input and stores it under the specified name  |
| `PAUSE`         | Wait for the user to press a key                       |

Strings can be interpolated using input key names prefixed with `$`.
