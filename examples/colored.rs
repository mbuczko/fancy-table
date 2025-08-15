use fancy_table::{
    Align, FancyTable, FancyTableOpts, Layout, Separator, TitleAlign, charset::Charset,
};

fn main() {
    let table = FancyTable::create(FancyTableOpts {
        charset: Charset::Classic,
        ..Default::default()
    })
    .add_title_with_align("props", TitleAlign::RightOffset(1))
    .add_column_named("ID", Layout::Slim)
    .add_column_named("NAME", Layout::Fixed(16))
    .add_column_named_wrapping_with_align("CHARACTER", Layout::Fixed(15), Align::Center)
    .add_column_named_with_align("BADNESS SCALE", Layout::Expandable(15), Align::Center)
    .add_column_named_wrapping_with_align("DESCRIPTION", Layout::Expandable(150), Align::Right)
    .hseparator(Some(Separator::Single))
    .padding(3)
    .build(102);

    table.render(vec![
        [
            "1",
            "\x1b[1mMaeglin\x1b[0m",
            "Elf",
            "Renegade\n10/10",
            "\x1b[31mMaeglin\x1b[0m is an elf who betrayed his fellow elves to the evil Morgoth in an age before \x1b[34mThe Lord of the Rings\x1b[0m.",
        ],
        [
            "29",
            "\x1b[1mTauriel\x1b[0m",
            "Woodland elf",
            "Tearjerker\n1/10",
            "\x1b[31mTauriel\x1b[0m is a woodland elf created for The Hobbit films. Her name means \x1b[33m\"daughter of the forest\"\x1b[0m in Sindarin.",
        ],
    ]);
}
