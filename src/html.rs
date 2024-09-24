use crate::{Header, Tasks};

pub fn generate_html(tasks: &Tasks, headers: &[Header], remark: &str) -> String {
    let div_header = "<div>";
    let div_foot = "</div>";
    let table_header = r#"<table border="0" cellspacing="1" cellpadding="0" width="1167" bgcolor=" #000000" height="14">"#;
    let table_foot = "</table>";
    let tr_header = r#"<tr bgcolor=" #ffffff" style="white-space: nowrap;">"#;
    let tr_foot = "</tr>";
    let mut html = format!("{}{}{}", div_header, table_header, tr_header);
    for (index, header) in headers.iter().enumerate() {
        if index == 0 {
            continue;
        }
        html.push_str(&format!(r#"<td bgcolor=" #dbeef3" height="14" style="padding: 5px;">
                <div align="left"><span
                        style="font-family: 宋体, serif, EmojiFont; color: rgb(0, 0, 0); font-size: 11px; font-weight: bold;">{}</span>
                </div>
            </td>"#, header.name));
    }

    html.push_str(tr_foot);
    html.push_str(tr_header);
    html.push_str(&format!(r#"<td bgcolor=" #ffffff" height="17" style="padding: 5px;">
                <div align="center"><span
                        style="font-family: 宋体, serif, EmojiFont; color: rgb(0, 0, 0); font-size: 15px;">{}</span>
                </div>
            </td>"#, tasks.email));
    html.push_str(&format!(r#"<td bgcolor=" #ffffff" height="17" style="padding: 5px;">
                <div align="center"><span
                        style="font-family: 宋体, serif, EmojiFont; color: rgb(0, 0, 0); font-size: 15px;">{}</span>
                </div>
            </td>"#, tasks.seq));
    html.push_str(&format!(r#"<td bgcolor=" #ffffff" height="17" style="padding: 5px;">
                <div align="center"><span
                        style="font-family: 宋体, serif, EmojiFont; color: rgb(0, 0, 0); font-size: 15px;">{}</span>
                </div>
            </td>"#, tasks.name));

    for task in tasks.info.iter() {
        html.push_str(&format!(r#"<td bgcolor=" #ffffff" height="17" style="padding: 5px;">
                <div align="center"><span
                        style="font-family: 宋体, serif, EmojiFont; color: rgb(0, 0, 0); font-size: 15px;">{}</span>
                </div>
            </td>"#, task));
    }
    format!("{}{}{}
    <p>&nbsp;</p>
    *附：{}{}", html, tr_foot, table_foot, remark, div_foot)
}