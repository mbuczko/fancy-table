# Why Fancy Table?
Initial motivation was to have an ASCII tables able to display multi-line rows, like JSON snippets. Although there are plenty of similar libraries in the wild, unfortunately none of them handled this case correctly. And this is what differs Fancy Table from competition in a first place.

Project evolved and during next weeks few other features got implemented to have tables even fancier:

- optional title at the top of table, aligned left or right
- per-column layouts - column may be specified with fixed/slim/expandale width
- per-column alignment - each column may be specified with its own alignment: left, right or center
- per column overflow behavior - each column may either truncate or wrap text which does not fit into given column width.
- customizable character sets: modern, classic, simple or minimal
- headers with customizable separator
- customizable (optional) row separators
- customizable padding

All exposed via simple, yet quite powerful API. For example, following code:

```rust
let mut table = FancyTable::create(FancyTableOpts {
       charset: Charset::Modern,
       ..Default::default()
   })
   .add_title_with_align("props", TitleAlign::RightOffset(1))
   .add_column_named("ID", Layout::Slim)
   .add_column_named("NAME", Layout::Fixed(16))
   .add_wrapping_column_named_with_align("CHARACTER", Layout::Fixed(11), Align::Center)
   .add_column_named_with_align("BADNESS SCALE", Layout::Expandable(15), Align::Center)
   .add_wrapping_column_named_with_align("DESCRIPTION", Layout::Expandable(150), Align::Right)
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

results in:

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
