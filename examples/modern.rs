use fancy_table::{Align, FancyTable, FancyTableOpts, Layout, Separator, TitleAlign};

fn main() {
    let mut table = FancyTable::create(FancyTableOpts::default())
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

    table.add_row(vec![
        "1",
        "Maeglin",
        "Elf",
        "Renegade\n10/10",
        "Maeglin is an elf who betrayed his fellow elves to the evil Morgoth in an age before The Lord of the Rings.",
    ]);
    table.add_row(vec![
        "29",
        "Tauriel",
        "Woodland elf",
        "Tearjerker\n1/10",
        "Tauriel is a woodland elf created for The Hobbit films. Her name means \"daughter of the forest\" in Sindarin.",
    ]);
    table.draw();
}
