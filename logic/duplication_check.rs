use ascent::ascent;

ascent! {
    // item(index, value)
    relation item(i32, String, String);

    // duplicated(value)
    relation duplicated(String, String);

    // If the same value appears at two different indices
    duplicated(v, scope) <--
        item(i1, v, scope),
        item(i2, v, scope),
        if i1 != i2;
}

pub fn find_duplicates(data: Vec<(i32, String, String)>) -> Vec<(String, String)> {
    let mut prog = AscentProgram::default();
    prog.item = data;
    prog.run();
    let r = prog.duplicated;
    r
}