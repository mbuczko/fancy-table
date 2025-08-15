# Why Fancy Table?
The primary motivation behind this project was creating ASCII tables capable of displaying multi-line rows, particularly for content like JSON snippets. While numerous similar libraries exist, none properly handled this use case—which is what distinguishes Fancy Table from its competitors.

The project has evolved significantly, with several additional features implemented to create even more sophisticated tables:

- Optional table titles with left or right alignment
- Flexible column layouts supporting fixed, slim, or expandable widths
- Individual column alignment options (left, right, or center)
- Configurable column overflow behavior (text truncation or wrapping)
- Multiple character set styles: modern, classic, simple, or minimal
- Customizable headers with adjustable separators
- Optional row separators with full customization
- Adjustable padding settings
- Handling ANSI escape codes in table content

## Installation

``` toml
[dependencies]
fancy-table = "0.3.1"
```

## Usage
All crucial functionality exposed via simple, yet quite powerful API:

```rust
let mut table = FancyTable::create(FancyTableOpts {
       charset: Charset::Modern,
       ..Default::default()
   })
   .add_title_with_align("props", TitleAlign::RightOffset(1))
   .add_column_named("ID", Layout::Slim)
   .add_column_named("NAME", Layout::Fixed(16))
   .add_column_named_wrapping_with_align("CHARACTER", Layout::Fixed(11), Align::Center)
   .add_column_named_with_align("BADNESS SCALE", Layout::Expandable(15), Align::Center)
   .add_column_named_wrapping_with_align("DESCRIPTION", Layout::Expandable(150), Align::Right)
   .padding(1)
   .hseparator(Some(Separator::Double))
   .rseparator(Some(Separator::Custom('┄')))
   .build(80);
    
table.render(vec![
    [
        "1",
        "Maeglin",
        "Elf",
        "Renegade\n10/10",
        "Maeglin is an elf who betrayed his fellow elves to the evil Morgoth in an age before The Lord of the Rings.",
    ],
    [
        "29",
        "Tauriel",
        "Woodland elf",
        "Tearjerker\n1/10",
        "Tauriel is a woodland elf created for The Hobbit films. Her name means \"daughter of the forest\" in Sindarin.",
    ]
]);
```

results in fancy looking table with title and headers:

```
╭────┬────────────────┬───────────┬───────────────┬──────────────────▪ props ▪─╮
│ ID │ NAME           │ CHARACTER │ BADNESS SCALE │                DESCRIPTION │
╞════╪════════════════╪═══════════╪═══════════════╪════════════════════════════╡
│ 1  │ Maeglin        │    Elf    │   Renegade    │      Maeglin is an elf who │
│    │                │           │     10/10     │  betrayed his fellow elves │
│    │                │           │               │  to the evil Morgoth in an │
├┄┄┄┄┼┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┼┄┄┄┄┄┄┄┄┄┄┄┼┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┼┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┤
│ 29 │ Tauriel        │ Woodland  │  Tearjerker   │  Tauriel is a woodland elf │
│    │                │    elf    │     1/10      │     created for The Hobbit │
│    │                │           │               │      films. Her name means │
╰────┴────────────────┴───────────┴───────────────┴────────────────────────────╯
```

Fanciness disclaimer: depending on your terminal font quality of final result may range from unreadable piece of sh*t to beautiful looking table :)

To get some more idea how tables may look like, have a look at examples:

```sh
# available examples: modern, classic, simple, minimal
cargo run --example modern
```
