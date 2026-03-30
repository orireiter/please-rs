# Please - A Shell
A shell intended to bring nice features from the UNIX/[OMZ](https://github.com/ohmyzsh/ohmyzsh) world into Windows.

# Installation
Currently it's possible to either use the binary in the latest release, or build it yourself. \
🚧 - An installer

# Features

Currently, in addition to achieving feature-parity with CMD (feedback is welcome!), <b>Please supports</b>:
1. History:
    * Search through historical commands with the up/down arrow, filtering by the current command as a prefix.
    * Persist historical commands in a `.please_history` file, both file location and amount of commands are configurable (See Configuration section below). 
2. Command completion by pressing `tab`:
    * A configurable list of completion providers is iterated through when pressing tab, stopping on the first match, and showing its candidates.<br>
    Example - a command starting with "git" will match the git provider, assuming no other custom provider has this match and is prior in the list.
    * Currently built-in providers are:
        * Directory Completion.
        * Git main commands completion.
        * Please commands.
        * 🚧 custom providers as configurations, are <u>not</u> supported yet.
3. Quick jumping and deletion with left/right/backspace + CTRL_C
4. Copying and pasting (already supported without requiring any implementation).
5. Customizable prefix elements
    * You can either use a built-in element (current dir, git), or create custom element. See the example configuration below, and specifically the `command.elements` part.
    * 🚧 The `shortened` and `home-relative` variants of the directory element are not yet implemented.
6. Configuration by JSON along a JSON schema. 
7. Aliasing of `ls` and `cat` to their Windows-equivalent.

# Configuration
Upon running, Please will [look for](https://github.com/orireiter/please-rs/blob/main/src/config.rs#L21) a `.please_config` in either the:

1. Currently open directory
2. The home directory

And create one in the home directory if neither exist.\
The config is in JSON format and you may find an example for it in the [latest release](https://github.com/orireiter/please-rs/releases/latest), as well as an auto-generated JSON schema.\
The Schema can be built by running `cargo run --bin generate_config_schema`.

<details>
<summary>An example config would be</summary>

```json
{
  "$schema": "https://github.com/orireiter/please-rs/releases/latest/download/please_config.schema.json",
  "command": {
    "prefix_config": {
      "prefix_to_command_delimiter": {
        "delimiter": " -> "
      },
      "prefix_elements_delimiter": {
        "delimiter": " | ",
        "color": "yellow"
      },
      "elements": [
        [
          {
            "Custom": {
              "command": "powershell",
              "args": [
                "get-date",
                "-F",
                "yyyy-MM-dd"
              ]
            }
          },
          {
            "display_parts": "ValueOnly",
            "color": "blue"
          }
        ],
        [
          {
            "Dir": "Full"
          },
          {
            "display_parts": "ValueOnly"
          }
        ],
        [
          "Git",
          {
            "display_parts": "ValueOnly",
            "color": "green"
          }
        ]
      ]
    },
    "completion_config": {
      "providers": [
        "Please",
        "Git",
        "Dir"
      ]
    }
  },
  "history": {
    "persistent_file": "C:\\Users\\<USER>\\.please_history",
    "max_commands_in_persistent_file": 1000
  }
}
```
</details>


---
<details>
<summary><b>Todos</b></summary>

1. Implement `&&`, `|` and other operators if any.
2. Support history with filtering. ✔️
3. Support basic auto completion by tab.  
3.1. and support omz plugin format.
4. Support customizations for:  
4.1. path prefix presentations.  
4.2. colors and such. ✔️
5. Implement some useful commands:
5.2 ls ✔️
5.3. cat ✔️
5.4. ...  
6. Support quick jumps with <arrow|delete> + ctrl ✔️
7. Validate why commands with `"` in them aren't interpreted correctly ✔️
8. Support quick clear of command with `ctrl+c`✔️
9. Handle copy+pasting ✔️ (was working out of the box I think)
10. Add git visual support if not part of omz plugin support. 
11. Add installation support to make usage easier
12. Allow marking text with shift+arrow
13. BUG - Handle bug where attempting to add new line if cursor is not at the end of the command, check backspace as well. ✔️
14. handle env vars before executable in command
</details>
