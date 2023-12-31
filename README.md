# Just another templating langauge (JTL)

> ⚠️WARNING⚠️ project actice development

```
Welcome {user.mention}, you're the {toPlacement | guild.count} to join!
```

A simple templating langauge built for dynamic text responces for any project but initally your discord bot.
Unlike many other templating langauges this one offers an LSP for your dashboard.

---

| Crate        | Description                                     | Progress |
| ------------ | ----------------------------------------------- | -------- |
| parser       | The core of the langauge                        | WIP 1    |
| runtime      | Processes the execution of a template           | WIP 2    |
| service      | Basic function for lsp                          | WIP 3    |
| lsp          | An orginized and operational lsp (server ready) | TODO? 5  |
| wasm_lsp     | An orginized and operational lsp (worker ready) | TODO? 6  |
| wasm_service | Simple wasm of service                          | WIP 4    |

<small>Numbers repersent the order in which I will be working though the project (help is appreciated)</small>

# Schema defintions (WIP)

```json
{
  "__version": "1.0.0",
  "functions": {
    "ToPlacement": {
      ":description": ["Converts number into placement"],
      "arguments": ["Int"],
      "return": "String"
    }
  },
  "structures": {
    "User": {
      ":description": [],
      "mention": ["String", "String mention of the member"]
    },
    "Guild": {
      ":description": [],
      "count": ["Int", "The number of member in the guild"],
      "name": ["String", null]
    }
  },
  "global": {
    "user": "#User",
    "guild": "#Guild",
    "toPlacement": "@ToPlacement"
  }
}
```

<small>In reference to the template provided initially</small>

# TODO (WIP)

- [ ] Bench test runtime, and parser, (maybe service...)
- [ ] Though out the code base update `Position` struct and rename `PPosition` as to not mix up with `Position` from 'lsp_types'
  - ```rust
    pub struct Position(pub usize, pub usize); // from
    pub struct Position { // to
      pub line: usize,
      pub character: usize
    };
    ```
- [ ] Fix major bug with resulting whitespace after tag and line breaks

  - Note: the position is consistent, however position calculation are not done to regain whitespace/line breaks.

  - ```rust
    let input = "Hay, {user.mention} \n welcome to {guild.name}";
    let expected = "Hay, <@xxx> \n welcome to BarFight";
    let input_return = "Hay, <@xxx>welcome to BarFight";

    let input = "Hay, {user.mention} welcome to {guild.name}";
    let expected = "Hay, <@xxx> welcome to BarFight";
    let input_return = "Hay, <@xxx>welcome to BarFight";
    ```
