use handlebars::Handlebars;
use itertools::Itertools;
use serde_json::json;

#[macro_use]
extern crate rust_embed;

#[derive(RustEmbed)]
#[folder = "./"]
struct Asset;

struct Line<'a> {
    text: &'a str,
    nr: usize,
}

impl Line<'_> {
    fn empty_line(&self) -> bool {
        self.text.trim().is_empty()
    }

    fn eq_txt(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

fn only_braces(pattern: &[Line]) -> bool {
    pattern.iter()
        .map(|p| p.text.chars())
        .flatten()
        .all(|c| c.is_whitespace() || ['{', '}'].contains(&c))
}

fn compare_lines(a: &[Line], b: &[Line]) -> bool {
    assert_eq!(a.len(), b.len());
    for (p, q) in a.iter().zip(b.iter()) {
        if !p.eq_txt(q) {
            return false;
        }
    }

    true
}

fn find_dup_lines(code: &str) -> Vec<Vec<(usize, usize)>> {
    let total_lines = code.split('\n').count();
    let owned_lines: Vec<_> = code.split('\n')
        .map(|s| s.trim().replace(" ", ""))
        .collect();

    let lines: Vec<Line> = owned_lines
        .iter()
        .map(|s| s.as_str())
        .enumerate()
        .map(|(i, l)| Line { text: l, nr: i })
        .filter(|l| !l.empty_line())
        .collect();
    let line_sl = lines.as_slice();

    let mut results = vec![];
    let mut used_lines = vec![false; total_lines];
    for w in (1..=(lines.len() / 2)).rev() {
        for pattern in line_sl.windows(w) {
            if pattern[0].empty_line() || used_lines[pattern[0].nr] || only_braces(pattern) {
                continue;
            }

            let starts = line_sl
                .windows(w)
                .filter(|p| compare_lines(p, pattern))
                .map(|p| (p[0].nr, p.last().unwrap().nr))
                .coalesce(|(a, b), (c, d)| {
                    if b < c {
                        Err(((a, b), (c, d)))
                    } else {
                        Ok((a, b))
                    }
                })
                .collect_vec();

            if starts.len() > 1 {
                // if any number in ranges in starts is used_lines, then skip.
                if starts
                    .iter()
                    .map(|&(a, b)| a..=b)
                    .flatten()
                    .any(|i| used_lines[i])
                {
                    continue;
                }
                for i in starts.iter().map(|&(a, b)| a..=b).flatten() {
                    used_lines[i] = true;
                }

                results.push(starts)
            }
        }
    }
    results
}

pub fn mark_dup_lines(s: &str) -> String {
    let index_html = Asset::get("index.hbs").unwrap();
    let template = std::str::from_utf8(index_html.as_ref()).unwrap();

    let spans = find_dup_lines(s);

    let data = json!({
        "lines": s.split('\n')
            .enumerate()
            .map(|(i, l)| {
                let highlight = !l.trim().is_empty() &&
                    spans.iter().flatten().find(|(f, l)| f <= &i && &i <= l).is_some();
                json!({
                    "line": l,
                    "highlight": highlight,
                })
            })
            .collect::<Vec<_>>()
    });

    let reg = Handlebars::new();
    reg.render_template(template, &data).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_ignore_whitespace() {
        assert_eq!(vec![vec![(0, 0), (1, 1)]], find_dup_lines(" aa=b;\n   aa  =b;\n"));
    }

    #[test]
    fn test_skip_braces() {
        let s = r#"
        {
        {
        }
        }
        {
        }
        {
        }
        }
        }
        "#;
        assert!(find_dup_lines(s).is_empty());
    }

    #[test]
    fn spans_overlap() {
        let s = r#"
        b
        c
        a
        b
        a
        b
        "#;
        assert_eq!(vec![vec![(3, 4), (5, 6)]], find_dup_lines(s));
    }

    #[test]
    fn multiple_spans() {
        let s = r#"
        a
        b
        c
        b
        a
        "#;
        assert_eq!(
            vec![vec![(1, 1), (5, 5)], vec![(2, 2), (4, 4)]],
            find_dup_lines(s)
        );
    }

    #[test]
    fn test_escapes() {
        let s = r#"
        a
        a<div>
        "#;

        assert!(!mark_dup_lines(s).contains("<div>"));
    }

    #[test]
    fn test_mark_lines() {
        let inp = r#"
        a a
        a a
        b
        "#;

        let outp = "<code class=\"rust\">\n<span class='hl'>        a a</span>\n<span class='hl'>        a a</span>\n        b\n        \n</code>\n";
        assert!(mark_dup_lines(inp).contains(outp));
    }

    #[test]
    fn test_ignores_empty_lines() {
        let s = r#"
        a

        a

        a
        a
        "#;
        assert_eq!(vec![vec![(1, 3), (5, 6)]], find_dup_lines(s));
    }

    #[test]
    fn test_group() {
        let s = r#"
        let a = 0;
        a += 1;
        a += 1;
        dbg(a)

        let a = 0;
        a += 1;
        a += 1;
        "#;
        assert_eq!(vec![vec![(1, 3), (6, 8)]], find_dup_lines(s));
    }

    #[test]
    fn test_3_occurences() {
        let s = r#"
        let a = 0;
        a += 1;
        dbg(a)

        let a = 0;
        a += 1;
        xxx;
        let a = 0;
        a += 1;
        "#;
        assert_eq!(vec![vec![(1, 2), (5, 6), (8, 9)]], find_dup_lines(s));
    }

    #[test]
    fn test_single_line() {
        let s = r#"
        let a = 0;

        let a = 0;
        "#;
        assert_eq!(vec![vec![(1, 1), (3, 3)]], find_dup_lines(s));
    }
}
