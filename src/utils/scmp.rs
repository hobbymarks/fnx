use ansi_term::{
    Color,
    Color::{Green, Red, RGB},
};
use difference::{Changeset, Difference};
use regex::Regex;
use tracing::trace;
use unicode_width::UnicodeWidthStr;

const SPACE_BOX: &str = "▯";
const EMPTY_BOX: &str = "␣";
const GRAY: Color = RGB(128, 128, 128);

fn noesc(s: &str) -> String {
    let re = Regex::new(r"\x1B\[([0-9;]+)m").unwrap();
    re.replace_all(s, "").to_string()
}

fn trace_noesc(sar: &str, origin: &str, edit: &str) {
    let noesc_origin = noesc(origin);
    let noesc_edit = noesc(edit);
    trace!("{}\n{:?}\n{:?}", sar, noesc_origin, noesc_edit);
}

pub fn s_compare(origin: &str, edit: &str, mode: &str) -> (String, String) {
    let changeset = Changeset::new(origin, edit, "");

    let mut c_origin = "".to_string();
    let mut c_edit = "".to_string();

    for diff in changeset.diffs {
        match diff {
            Difference::Same(s) => {
                let s = s.replace(char::is_whitespace, SPACE_BOX);

                let noesc_origin = noesc(&c_origin);
                let noesc_edit = noesc(&c_edit);
                trace!("Sam\n{:?}\n{:?}", noesc_origin, noesc_edit);

                //alignment origin and edited length by fill with EMPTY_BOX
                if mode == "a" {
                    match noesc_origin.width().cmp(&noesc_edit.width()) {
                        std::cmp::Ordering::Less => {
                            let fill = EMPTY_BOX.repeat(noesc_edit.width() - noesc_origin.width());
                            c_origin.push_str(&GRAY.paint(fill).to_string());
                        }
                        std::cmp::Ordering::Equal => {}
                        std::cmp::Ordering::Greater => {
                            let dif_len = noesc_origin.width() - noesc_edit.width();

                            let fill = EMPTY_BOX.repeat(dif_len);
                            c_edit.push_str(&GRAY.paint(fill).to_string());
                        }
                    }
                }

                c_origin.push_str(&s);
                c_edit.push_str(&s);

                trace_noesc("", &c_origin, &c_edit);
            }
            Difference::Add(s) => {
                trace_noesc("Add", &c_origin, &c_edit);

                let s = s.replace(char::is_whitespace, SPACE_BOX);
                c_edit.push_str(&Green.paint(s).to_string());

                trace_noesc("", &c_origin, &c_edit);
            }
            Difference::Rem(s) => {
                trace_noesc("Rem", &c_origin, &c_edit);

                let s = s.replace(char::is_whitespace, SPACE_BOX);
                c_origin.push_str(&Red.paint(s).to_string());

                trace_noesc("", &c_origin, &c_edit);
            }
        }
    }

    let noesc_origin = noesc(&c_origin);
    let noesc_edit = noesc(&c_edit);
    trace!("TailSam\n{:?}\n{:?}", noesc_origin, noesc_edit);

    //alignment origin and edited length by fill with EMPTY_BOX
    if mode == "a" {
        match noesc_origin.width().cmp(&noesc_edit.width()) {
            std::cmp::Ordering::Less => {
                let fill = EMPTY_BOX.repeat(noesc_edit.width() - noesc_origin.width());
                c_origin.push_str(&GRAY.paint(fill).to_string());
            }
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => {
                let dif_len = noesc_origin.width() - noesc_edit.width();

                let fill = EMPTY_BOX.repeat(dif_len);
                c_edit.push_str(&GRAY.paint(fill).to_string());
            }
        }
    }

    (c_origin, c_edit)
}

#[cfg(test)]
mod tests {
    use crate::utils::s_compare;

    #[test]
    fn test_s_compare() {
        let origin = "A B C";
        let origin_a = "A\u{1b}[31m▯\u{1b}[0mB\u{1b}[31m▯\u{1b}[0mC";
        let edit = "A_B_C";
        let edit_a = "A\u{1b}[32m_\u{1b}[0mB\u{1b}[32m_\u{1b}[0mC";
        let (o_r, e_r) = s_compare(origin, edit, "a");
        assert_eq!(origin_a, o_r);
        assert_eq!(edit_a, e_r);
    }
}
