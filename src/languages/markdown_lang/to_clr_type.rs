use std::collections::HashMap;

use parser_lib::markdown_lang::{
    StringValueTable, Table, load_md_file, load_md_file_from_str,
};

pub(crate) fn md_table_to_string_row_table(file_name: &str) -> Result<StringValueTable, String> {
    let md_file = load_md_file(file_name).map_err(|e| format!("{e:?}"))?;
    let tables = md_file.get_tables();
    let Some(table) = tables.first() else {
        return Ok(StringValueTable::default());
    };

    let mut rows = Vec::new();
    for row in table.data_rows() {
        let mut out_row = Vec::new();
        for cell in row {
            out_row.push(cell.get_text().trim().to_string());
        }
        rows.push(out_row);
    }

    Ok(StringValueTable::new(rows))
}

pub(crate) fn md_table_to_hash2(file_content: &str) -> Result<HashMap<String, String>, String> {
    let md_file = load_md_file_from_str(file_content).map_err(|e| format!("{e:?}"))?;
    let tables = md_file.get_tables();
    let Some(table) = tables.first() else {
        return Ok(HashMap::default());
    };

    md_table_object_to_hash(table)
}

pub(crate) fn md_table_object_to_hash(table: &Table) -> Result<HashMap<String, String>, String> {
    let mut out = HashMap::default();
    let col_count = table.get_col_names().map_err(|e| format!("{e:?}"))?.len();
    if col_count != 2 {
        return Err(format!("expected 2 columns, got {col_count}"));
    }

    for row in table.data_rows() {
        let key = row[0].get_text().trim().to_string();
        let value = row[1].get_text().trim().to_string();
        if !value.is_empty() {
            out.insert(key, value);
            continue;
        }

        let link_value = row[1]
            .get_inline_items()
            .first()
            .and_then(|x| x.get_link())
            .map(|x| x.get_link_text().trim().to_string())
            .unwrap_or_default();
        out.insert(key, link_value);
    }

    Ok(out)
}
