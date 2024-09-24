use std::path::PathBuf;
use calamine::{open_workbook, DataType, Reader, Xls, Xlsx};
use crate::{Header, Tasks};

pub fn parse_excel(path: PathBuf) -> (Vec<Tasks>, Vec<Header>) {
    let ext = path.extension().unwrap();
    let mut headers = vec![];
    let mut tasks_list = vec![];
    if ext == "xlsx" {
        let mut workbook: Xlsx<_> = open_workbook(path).expect("Failed to open Excel file");
    } else if ext == "xls" {
        let mut workbook: Xls<_> = open_workbook(path).expect("Failed to open Excel file");
        if let Ok(range) = workbook.worksheet_range("Sheet1") {
            let mut first_line = true;
            for row in range.rows() {
                if first_line {
                    headers = row.iter()
                        .map(|cell| {
                            let mut name = cell.to_string();
                            name = name.replace("\n", "");
                            let mut width = 100.0;
                            if name == "邮箱地址" {
                                width = 250.0
                            }
                            Header { name, width, check: true }
                        }).collect();
                    first_line = false;
                    headers.insert(0, Header { name: "全选".to_owned(), width: 50.0, check: true });
                    continue;
                }
                let mut tasks = Tasks::default();
                let mut info = vec! {};
                for (i, v) in row.iter().enumerate() {
                    if i == 0 {
                        tasks.email = v.to_string();
                    } else if i == 1 {
                        if v.is_int() {
                            tasks.seq = v.get_int().unwrap_or(0);
                        } else if v.is_float() {
                            tasks.seq = v.get_float().unwrap_or(0f64) as i64;
                        } else {
                            tasks.seq = 0;
                        }
                    } else if i == 2 {
                        tasks.name = v.to_string();
                    } else {
                        info.push(format!("{}", v));
                    }
                }
                tasks.info = info;
                tasks.status = true;
                tasks_list.push(tasks);
            }
        }
    }
    (tasks_list, headers)
}