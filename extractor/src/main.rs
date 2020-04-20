use anyhow::{anyhow, Result};
use parse_wiki_text::{Configuration, Node};

fn get_content(cell: &parse_wiki_text::TableCell) -> Option<String> {
    return Some(if let Node::Text{value,..} = cell.content.get(0)? {
        value.to_string()
    } else {
        String::new()
    });
}

fn count(cells: &Vec<parse_wiki_text::TableCell>) -> usize {
    let mut u = 0;
    for cell in cells {
        u += 1;
        if let Some(e) = &cell.attributes {
            for l in e {
                if let Node::Text { value, ..} = l {
                    if value.contains("colspan=") {
                        u += value.split("\"").skip(1).next().expect("A").parse::<usize>().expect("B")-1;
                    }
                }
            }
        }
    }
    u
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
struct Row {
    cells: Vec<String>
}

impl Row {

    fn new(vec: Vec<String>) -> Self {
        Self {
            cells: vec
        }
    }
}



#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
struct Table {
    rows: Vec<Row>
}

impl Table {

    fn new(rows: &std::vec::Vec<parse_wiki_text::TableRow<'_>>) -> Self {
        let mut p = 0;
        for row in rows {
            let k = count(&row.cells);
            if p < k {
                p = k;
            }
        }
        let mut table = (0..rows.len()).map(|_| {
            (0..p).map(|_| {
                "".to_string()
            }).collect::<Vec<String>>()
        }).collect::<Vec<Vec<String>>>();
        
        for (y,row) in rows.iter().enumerate() {
            let mut x = 0;
            for cell in row.cells.iter() {
                let content = if let Some(e) = get_content(cell) {
                    if e.is_empty() {
                        "<empty>".to_owned()
                    } else {
                        e.clone()
                    }
                } else {
                    "<empty>".to_owned()
                };
                while !table[y][x].is_empty() {
                    x += 1;
                    
                }
                table[y][x] = content.clone();
                if let Some(e) = &cell.attributes {
                    for l in e {
                        if let Node::Text { value, ..} = l {
                            if value.contains("rowspan=") {
                                for p in 0..value.split("\"").skip(1).next().expect("D").parse::<usize>().expect("C") {
                                    table[y+p][x] = content.clone();
                                }
                            } else if value.contains("colspan=") {
                                for p in 0..value.split("\"").skip(1).next().expect("A").parse::<usize>().expect("B")-1 {
                                    table[y][x+p] = content.clone();
                                }
                            }
                        }
                    }
                }
                x += 1;
            }

        }
        //println!("{:#?}",table);

        let mut rows: Vec<Row> = Vec::with_capacity(table.len());
        for t in table {
            rows.push(Row::new(t));
        }
        return Self {
            rows
        }
    }
}

struct Tree {
    value: String,
    childs: Vec<Tree>
}

impl Tree {
    /*
    a a a b b a a a 
    h j i m o p o e
    h j i m o p o e
    h j i m o p o e
    h j i m o p o e
    */

    fn new(table: &Table) {
        let mut tree_map = Vec::new();

        println!("-----------------------");

        for (x,cell) in table.rows[0].cells.iter().enumerate() {
            if cell == "Field Name" {
                let mut temp = Vec::new();
                for row in &table.rows {
                    temp.push(row.cells[x].to_string());
                }
                tree_map.push(temp);
            }
        }

        println!("{:?}",tree_map);
    }
}

fn main() -> Result<()> {
    let response = fetch_wiki_page("https://wiki.vg/api.php", "Protocol")?;
    let wiki_text = response["parse"]["wikitext"]["*"]
        .as_str()
        .ok_or(anyhow!("wikitext not present"))?;

    let mut tmap: Vec<Tree> = Vec::new();

    let result = create_configuration().parse(wiki_text);

    result
        .nodes
        .iter()
        .filter_map(|n| match n {
            Node::Table { rows, .. } => rows
                .get(0)
                .and_then(|r| r.cells.get(0))
                .and_then(|c| c.content.get(0))
                .filter(|n| match n {
                    Node::Text {
                        value: "Packet ID", ..
                    } => true,
                    _ => false,
                })
                .and(Some(rows)),
            _ => None,
        })
        .for_each(|n| {
            Tree::new(&Table::new(n));
            /*n.iter().for_each(|row| {
                println!("{:#?}", row);
            });*/
        });
    Ok(())
}

fn fetch_wiki_page(url: &str, page: &str) -> Result<serde_json::Value> {
    let url = reqwest::Url::parse_with_params(
        url,
        &[
            ("action", "parse"),
            ("page", page),
            ("format", "json"),
            ("prop", "wikitext"),
        ],
    )?;
    let response = reqwest::blocking::get(url)?;
    Ok(response.json::<serde_json::Value>()?)
}

fn create_configuration() -> ::parse_wiki_text::Configuration {
    Configuration::new(&parse_wiki_text::ConfigurationSource {
        category_namespaces: &["category"],
        extension_tags: &[
            "gallery",
            "indicator",
            "nowiki",
            "pre",
            "ref",
            "references",
            "source",
            "syntaxhighlight",
        ],
        file_namespaces: &["file", "image"],
        link_trail: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
        magic_words: &[
            "EXPECTUNUSEDCATEGORY",
            "FORCETOC",
            "HIDDENCAT",
            "INDEX",
            "NEWSECTIONLINK",
            "NOCC",
            "NOCONTENTCONVERT",
            "NOEDITSECTION",
            "NOGALLERY",
            "NOINDEX",
            "NONEWSECTIONLINK",
            "NOTC",
            "NOTITLECONVERT",
            "NOTOC",
            "STATICREDIRECT",
            "TOC",
        ],
        protocols: &[
            "//",
            "bitcoin:",
            "ftp://",
            "ftps://",
            "geo:",
            "git://",
            "gopher://",
            "http://",
            "https://",
            "irc://",
            "ircs://",
            "magnet:",
            "mailto:",
            "mms://",
            "news:",
            "nntp://",
            "redis://",
            "sftp://",
            "sip:",
            "sips:",
            "sms:",
            "ssh://",
            "svn://",
            "tel:",
            "telnet://",
            "urn:",
            "worldwind://",
            "xmpp:",
        ],
        redirect_magic_words: &["REDIRECT"],
    })
}
